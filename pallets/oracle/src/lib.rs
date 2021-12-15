#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
  use super::*;
  use frame_support::{
    inherent::Vec,
    pallet_prelude::*,
    traits::{
      fungibles::{Inspect, Mutate, Transfer},
      tokens::WithdrawConsequence,
    },
    PalletId,
  };
  use frame_system::pallet_prelude::*;
  use sp_runtime::traits::AccountIdConversion;
  use tidefi_primitives::{
    pallet::{FeesExt, OracleExt, SecurityExt},
    AssetId, Balance, CurrencyId, Hash, Trade, TradeConfirmation, TradeStatus,
  };

  /// Oracle configuration
  #[pallet::config]
  pub trait Config:
    frame_system::Config + pallet_assets::Config<AssetId = AssetId, Balance = Balance>
  {
    /// Events
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    /// Pallet ID
    #[pallet::constant]
    type OraclePalletId: Get<PalletId>;

    /// Weights
    type WeightInfo: WeightInfo;

    /// Security traits
    type Security: SecurityExt<Self::AccountId, Self::BlockNumber>;

    /// Fees traits
    type Fees: FeesExt<Self::AccountId>;

    /// Currency wrapr
    type CurrencyWrapr: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  /// Oracle is enabled
  #[pallet::storage]
  #[pallet::getter(fn status)]
  pub(super) type OracleStatus<T: Config> = StorageValue<_, bool, ValueQuery>;

  /// Oracle Account ID
  #[pallet::storage]
  #[pallet::getter(fn account_id)]
  pub type OracleAccountId<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

  /// Mapping of pending Trades
  #[pallet::storage]
  #[pallet::getter(fn trades)]
  pub type Trades<T: Config> =
    StorageMap<_, Blake2_128Concat, Hash, Trade<T::AccountId, T::BlockNumber>>;

  /// Genesis configuration
  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    /// Oracle status
    pub enabled: bool,
    /// Oracle Account ID. Multisig is supported.
    /// This account will be able to confirm trades on-chain.
    pub account: T::AccountId,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        // Oracle is enabled by default
        enabled: true,
        // We use pallet account ID by default,
        // but should always be set in the genesis config.
        account: T::OraclePalletId::get().into_account(),
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      OracleStatus::<T>::put(self.enabled);
      OracleAccountId::<T>::put(self.account.clone());
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Oracle status changed
    StatusChanged { is_enabled: bool },
    /// Oracle account changed
    AccountChanged { account_id: T::AccountId },
    /// Oracle confirmed trade
    Traded {
      request_id: Hash,
      status: TradeStatus,
      account_id: T::AccountId,
      currency_from: CurrencyId,
      currency_amount_from: Balance,
      currency_to: CurrencyId,
      currency_amount_to: Balance,
    },
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// The Quorum is paused. Try again later.
    OraclePaused,
    /// The access to the Oracle pallet is not allowed for this account ID.
    AccessDenied,
    /// Invalid request ID.
    InvalidRequestId,
    /// Invalid request status.
    InvalidRequestStatus,
    /// Invalid market maker status.
    InvalidMarketMakerRequest,
    /// There is a conflict in the request.
    Conflict,
    /// Unable to transfer token.
    TransferFailed,
    /// Unable to burn token.
    BurnFailed,
    /// Unable to mint token.
    MintFailed,
    /// Unable to take or calculate network fees.
    FeesFailed,
    /// Unknown Asset.
    UnknownAsset,
    /// No Funds available for this Asset Id.
    NoFunds,
    /// Trade overflow
    Overflow,
    /// MarketMakers do not have enough funds
    MarketMakerNoFunds,
    /// MarketMakers cannot deposit source funds of the trade
    MarketMakerCantDeposit,
    /// Unknown Error.
    UnknownError,
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Oracle have confirmation and confirm the trade.
    ///
    /// - `request_id`: Unique request ID.
    /// - `market_makers`: Vector of `TradeConfirmation` who represent the allocation of multiple source.
    ///
    /// Emits `Traded` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::confirm_trade())]
    pub fn confirm_trade(
      origin: OriginFor<T>,
      request_id: Hash,
      market_makers: Vec<TradeConfirmation>,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the oracle/chain is not paused
      Self::ensure_not_paused()?;

      // 2. Make sure this is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(sender == Self::account_id(), Error::<T>::AccessDenied);

      // 3. Make sure the `request_id` exist
      Trades::<T>::try_mutate_exists(request_id, |trade| {
        match trade {
          None => {
            return Err(Error::<T>::InvalidRequestId);
          }
          Some(trade) => {
            // 5. Make sure the trade status is pending or partially filled
            if trade.status != TradeStatus::Pending && trade.status != TradeStatus::PartiallyFilled
            {
              return Err(Error::<T>::InvalidRequestStatus);
            }

            // 6. Calculate totals and all market makers
            let mut total_from: Balance = 0;
            let mut total_to: Balance = 0;

            for mm in market_makers.iter() {
              let mm_trade_request = Trades::<T>::try_get(mm.request_id)
                .map_err(|_| Error::<T>::InvalidMarketMakerRequest)?;

              // make sure all the market markers have enough funds before we can continue
              match T::CurrencyWrapr::can_withdraw(
                trade.token_to,
                &mm_trade_request.account_id,
                mm.amount_to_send,
              ) {
                // make sure we can deposit
                WithdrawConsequence::Success => {
                  T::CurrencyWrapr::can_deposit(
                    trade.token_from,
                    &mm_trade_request.account_id,
                    mm.amount_to_receive,
                  )
                  .into_result()
                  .map_err(|_| Error::<T>::MarketMakerCantDeposit)?;

                  // alls good, let's calculate our totals
                  total_from += mm.amount_to_receive;
                  total_to += mm.amount_to_send;
                }
                // no funds error
                WithdrawConsequence::NoFunds => return Err(Error::MarketMakerNoFunds),
                // unknown assets
                WithdrawConsequence::UnknownAsset => return Err(Error::UnknownAsset),
                // throw an error, we really need a success here
                _ => return Err(Error::UnknownError),
              }
            }

            // 7. a) Validate totals
            trade.amount_from_filled += total_from;
            trade.amount_to_filled += total_to;

            if total_from < trade.amount_from {
              trade.status = TradeStatus::PartiallyFilled;
              // trade overflow
              if trade.amount_from_filled > trade.amount_from {
                return Err(Error::Overflow);
              }
            }

            // FIXME: Add slippage check for partial fill
            // the best would probably get the average value for 1 token, then do the check for 1:1 with provided slippage calculation
            // 7. b) Maximum of 10% slippage for the `amount_to`
            //if trade.amount_to_filled * 10 < trade.amount_to * 9 || trade.amount_to_filled * 10 > trade.amount_to * 11 {
            //  return Err(Error::Conflict);
            //}

            // 8. Make sure the `account_id` can withdraw the funds
            match T::CurrencyWrapr::can_withdraw(
              trade.token_from,
              &trade.account_id,
              trade.amount_from,
            ) {
              WithdrawConsequence::Success => {
                // 9. Calculate and transfer network fee
                // FIXME: Should we take a transfer fee on the FROM or the TO asset or both?
                let _trading_fees = T::Fees::calculate_trading_fees(trade.token_to, total_to);

                // 10. Make sure the requester can deposit the new asset before initializing trade process
                T::CurrencyWrapr::can_deposit(trade.token_to, &trade.account_id, total_to)
                  .into_result()
                  .map_err(|_| Error::<T>::BurnFailed)?;

                for mm in market_makers.iter() {
                  Trades::<T>::try_mutate_exists(mm.request_id, |mm_trade_request| {
                    match mm_trade_request {
                      None => Err(Error::<T>::InvalidRequestId),
                      Some(market_maker_trade_intent) => {
                        // 11. a) Make sure the marketmaker trade request is still valid
                        if market_maker_trade_intent.status != TradeStatus::Pending
                          && market_maker_trade_intent.status != TradeStatus::PartiallyFilled
                        {
                          return Err(Error::<T>::InvalidRequestStatus);
                        }

                        // 11. b) Make sure the currency match
                        if market_maker_trade_intent.token_from != trade.token_to {
                          return Err(Error::<T>::InvalidMarketMakerRequest);
                        }

                        // 11. c) make sure market maker have enough funds in the trade intent request
                        if (market_maker_trade_intent.amount_from
                          - market_maker_trade_intent.amount_from_filled)
                          < mm.amount_to_send
                        {
                          return Err(Error::<T>::InvalidMarketMakerRequest);
                        }

                        if mm.amount_to_send
                          < (market_maker_trade_intent.amount_from
                            - market_maker_trade_intent.amount_from_filled)
                        {
                          // partial fill
                          market_maker_trade_intent.status = TradeStatus::PartiallyFilled;
                        }

                        market_maker_trade_intent.amount_from_filled += mm.amount_to_send;
                        market_maker_trade_intent.amount_to_filled += mm.amount_to_receive;

                        if market_maker_trade_intent.amount_from
                          == market_maker_trade_intent.amount_from_filled
                        {
                          market_maker_trade_intent.status = TradeStatus::Completed;
                        }

                        // 11. d) Transfer funds from the requester to the market makers
                        let amount_and_fee =
                          T::Fees::calculate_trading_fees(trade.token_from, mm.amount_to_receive);
                        if T::CurrencyWrapr::transfer(
                          trade.token_from,
                          &trade.account_id,
                          &market_maker_trade_intent.account_id,
                          // deduce the fee from the amount
                          mm.amount_to_receive - amount_and_fee.fee,
                          true,
                        )
                        .is_err()
                        {
                          // FIXME: Add rollback
                        }

                        // 11. e) Requester pay fees of the transaction, but this is deducted
                        // from the MM final amount, so this is paid by the MM
                        if T::CurrencyWrapr::transfer(
                          trade.token_from,
                          &trade.account_id,
                          &T::Fees::account_id(),
                          amount_and_fee.fee,
                          true,
                        )
                        .is_err()
                        {
                          // FIXME: Add rollback
                        }

                        // 11. f) Register a new trading fees associated with the account.
                        // A percentage of the network profits will be re-distributed to the account at the end of the era.
                        T::Fees::register_trading_fees(
                          trade.account_id.clone(),
                          trade.token_from,
                          mm.amount_to_receive,
                        );

                        // 12. a) Transfer funds from the market makers to the account
                        let amount_and_fee =
                          T::Fees::calculate_trading_fees(trade.token_to, mm.amount_to_send);

                        if T::CurrencyWrapr::transfer(
                          trade.token_to,
                          &market_maker_trade_intent.account_id,
                          &trade.account_id,
                          // deduce the fee from the amount
                          mm.amount_to_send - amount_and_fee.fee,
                          true,
                        )
                        .is_err()
                        {
                          // FIXME: Add rollback
                        }

                        // 12. b) Market makers pay fees of the transaction, but this is deducted
                        // from the requester final amount, so this is paid by the requester
                        if T::CurrencyWrapr::transfer(
                          trade.token_to,
                          &market_maker_trade_intent.account_id,
                          &T::Fees::account_id(),
                          amount_and_fee.fee,
                          true,
                        )
                        .is_err()
                        {
                          // FIXME: Add rollback
                        }

                        // 12. c) Register a new trading fees associated with the account.
                        // A percentage of the network profits will be re-distributed to the account at the end of the era.
                        T::Fees::register_trading_fees(
                          market_maker_trade_intent.account_id.clone(),
                          trade.token_to,
                          mm.amount_to_send,
                        );

                        Ok(())
                      }
                    }
                  })?;
                }

                // close the trade if it's complete (we don't use the amount_to compare as there is a slippage to validate)
                if trade.amount_from == trade.amount_from_filled {
                  trade.status = TradeStatus::Completed;
                }

                // 13. Emit event on chain
                Self::deposit_event(Event::<T>::Traded {
                  request_id,
                  status: trade.status.clone(),
                  account_id: trade.account_id.clone(),
                  currency_from: trade.token_from,
                  currency_amount_from: total_from,
                  currency_to: trade.token_to,
                  currency_amount_to: total_to,
                });
              }
              WithdrawConsequence::NoFunds => return Err(Error::<T>::NoFunds),
              WithdrawConsequence::UnknownAsset => return Err(Error::<T>::UnknownAsset),
              _ => return Err(Error::<T>::UnknownError),
            };
          }
        }

        Ok(())
      })?;

      Ok(().into())
    }

    /// Oracle change the account ID who can confirm trade.
    ///
    /// Make sure to have access to the `account_id` otherwise
    /// only `root` will be able to update the oracle account.
    ///
    /// - `new_account_id`: The new Oracle account id.
    ///
    /// Emits `AccountChanged` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::set_account_id())]
    pub fn set_account_id(
      origin: OriginFor<T>,
      new_account_id: T::AccountId,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure this is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(sender == Self::account_id(), Error::<T>::AccessDenied);

      // 2. Update oracle account
      OracleAccountId::<T>::put(new_account_id.clone());

      // 3. Emit event on chain
      Self::deposit_event(Event::<T>::AccountChanged {
        account_id: new_account_id,
      });

      Ok(().into())
    }

    /// Change Oracle status.
    ///
    /// - `is_enabled`: Is the oracle enabled?
    ///
    /// Emits `StatusChanged` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::set_status())]
    pub fn set_status(origin: OriginFor<T>, is_enabled: bool) -> DispatchResultWithPostInfo {
      // 1. Make sure this is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(sender == Self::account_id(), Error::<T>::AccessDenied);

      // 2. Update oracle status
      OracleStatus::<T>::set(is_enabled);

      // 3. Emit event on chain
      Self::deposit_event(Event::<T>::StatusChanged { is_enabled });

      Ok(().into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config> Pallet<T> {
    fn ensure_not_paused() -> Result<(), DispatchError> {
      if Self::is_oracle_enabled() {
        Ok(())
      } else {
        Err(Error::<T>::OraclePaused.into())
      }
    }
  }

  // implement the `OracleExt` functions
  impl<T: Config> OracleExt<T::AccountId, T::BlockNumber> for Pallet<T> {
    fn is_oracle_enabled() -> bool {
      // make sure the chain and the oracle pallet are enabled
      T::Security::is_chain_running() && Self::status()
    }

    fn add_new_trade_in_queue(
      account_id: T::AccountId,
      asset_id_from: CurrencyId,
      amount_from: Balance,
      asset_id_to: CurrencyId,
      amount_to: Balance,
    ) -> (Hash, Trade<T::AccountId, T::BlockNumber>) {
      // unique request id
      let request_id = T::Security::get_unique_id(account_id.clone());
      let trade = Trade {
        account_id,
        token_from: asset_id_from,
        token_to: asset_id_to,
        amount_from,
        amount_from_filled: 0,
        amount_to,
        amount_to_filled: 0,
        status: TradeStatus::Pending,
        block_number: <frame_system::Pallet<T>>::block_number(),
      };

      // insert in our queue
      Trades::<T>::insert(request_id, trade.clone());

      // return values
      (request_id, trade)
    }
  }
}
