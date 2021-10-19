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
    /// Oracle status changed \[is_enabled\]
    StatusChanged(bool),
    /// Oracle account changed \[account_id\]
    AccountChanged(T::AccountId),
    /// Oracle confirmed trade
    /// \[request_id, account_id, token_from, token_amount_from, token_to, token_amount_to\]
    Traded(Hash, T::AccountId, CurrencyId, Balance, CurrencyId, Balance),
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
      market_makers: Vec<TradeConfirmation<T::AccountId>>,
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
            // 4. Make sure the trade status is pending
            if trade.status != TradeStatus::Pending {
              return Err(Error::<T>::InvalidRequestStatus);
            }

            // 5. Calculate totals and all market makers
            let mut total_from: Balance = 0;
            let mut total_to: Balance = 0;

            for mm in market_makers.iter() {
              // make sure all the market markers have enough funds before we can continue
              match T::CurrencyWrapr::can_withdraw(
                trade.token_to,
                &mm.account_id,
                mm.amount_to_send,
              ) {
                // make sure we can deposit
                WithdrawConsequence::Success => {
                  T::CurrencyWrapr::can_deposit(
                    trade.token_from,
                    &mm.account_id,
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

            // 6. a) Validate totals
            if trade.amount_from != total_from {
              return Err(Error::<T>::Conflict);
            }
            // 6. b) Maximum of 10% slippage for the `amount_to`
            if total_to * 10 < trade.amount_to * 9 || total_to * 10 > trade.amount_to * 11 {
              return Err(Error::Conflict);
            }

            // 7. Make sure the `account_id` can withdraw the funds
            match T::CurrencyWrapr::can_withdraw(
              trade.token_from,
              &trade.account_id,
              trade.amount_from,
            ) {
              WithdrawConsequence::Success => {
                // 8. Calculate and transfer network fee
                // FIXME: Should we take a transfer fee on the FROM or the TO asset or both?
                let trading_fees = T::Fees::calculate_trading_fees(trade.token_to, total_to);

                // 9. Make sure the requester can deposit the new asset before initializing trade process
                T::CurrencyWrapr::can_deposit(trade.token_to, &trade.account_id, total_to)
                  .into_result()
                  .map_err(|_| Error::<T>::BurnFailed)?;

                for mm in market_makers.iter() {
                  // 10. a) Transfer funds from the requester to the market makers
                  let amount_and_fee =
                    T::Fees::calculate_trading_fees(trade.token_from, mm.amount_to_receive);
                  if T::CurrencyWrapr::transfer(
                    trade.token_from,
                    &trade.account_id,
                    &mm.account_id,
                    // deduce the fee from the amount
                    mm.amount_to_receive - amount_and_fee.fee,
                    true,
                  )
                  .is_err()
                  {
                    // FIXME: Add rollback
                  }

                  // 10. b) Requester pay fees of the transaction, but this is deducted
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

                  // 10. c) Register a new trading fees associated with the account.
                  // A percentage of the network profits will be re-distributed to the account at the end of the era.
                  T::Fees::register_trading_fees(
                    trade.account_id.clone(),
                    trade.token_from,
                    mm.amount_to_receive,
                  );

                  // 11. a) Transfer funds from the market makers to the account
                  let amount_and_fee =
                    T::Fees::calculate_trading_fees(trade.token_to, mm.amount_to_send);
                  if T::CurrencyWrapr::transfer(
                    trade.token_to,
                    &mm.account_id,
                    &trade.account_id,
                    // deduce the fee from the amount
                    mm.amount_to_send - amount_and_fee.fee,
                    true,
                  )
                  .is_err()
                  {
                    // FIXME: Add rollback
                  }

                  // 11. b) Market makers pay fees of the transaction, but this is deducted
                  // from the requester final amount, so this is paid by the requester
                  if T::CurrencyWrapr::transfer(
                    trade.token_to,
                    &mm.account_id,
                    &T::Fees::account_id(),
                    amount_and_fee.fee,
                    true,
                  )
                  .is_err()
                  {
                    // FIXME: Add rollback
                  }

                  // 11. c) Register a new trading fees associated with the account.
                  // A percentage of the network profits will be re-distributed to the account at the end of the era.
                  T::Fees::register_trading_fees(
                    mm.account_id.clone(),
                    trade.token_to,
                    mm.amount_to_send,
                  );
                }

                // 13. Emit event on chain
                Self::deposit_event(Event::<T>::Traded(
                  request_id,
                  trade.account_id.clone(),
                  trade.token_from,
                  trade.amount_from,
                  trade.token_to,
                  trade.amount_to,
                ));
              }
              WithdrawConsequence::NoFunds => return Err(Error::<T>::NoFunds),
              WithdrawConsequence::UnknownAsset => return Err(Error::<T>::UnknownAsset),
              _ => return Err(Error::<T>::UnknownError),
            };
          }
        }

        // it deletes the item if mutated to a None.
        // FIXME: do we want to keep a copy and update the status to completed?
        *trade = None;
        Ok(())
      })?;

      Ok(().into())
    }

    // FIXME: [@lemarier] Should be removed after the demo.
    //
    /// Quick trade for demo.
    ///
    /// - `account_id`: Account ID.
    /// - `asset_id_from`: Asset Id to send.
    /// - `amount_from`: Amount to send.
    /// - `asset_id_to`: Asset Id to receive.
    /// - `amount_to`: Amount to receive.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::quick_trade())]
    pub fn quick_trade(
      origin: OriginFor<T>,
      account_id: T::AccountId,
      asset_id_from: CurrencyId,
      amount_from: Balance,
      asset_id_to: CurrencyId,
      amount_to: Balance,
    ) -> DispatchResultWithPostInfo {
      // make sure the quorum is not paused
      Self::ensure_not_paused()?;

      // make sure it's the oracle
      let sender = ensure_signed(origin)?;
      ensure!(sender == Self::account_id(), Error::<T>::AccessDenied);

      // make sure the FROM balance is available
      match T::CurrencyWrapr::can_withdraw(asset_id_from, &account_id, amount_from) {
        WithdrawConsequence::Success => {
          // make sure we can deposit before burning
          T::CurrencyWrapr::can_deposit(asset_id_to, &account_id, amount_to)
            .into_result()
            .map_err(|_| Error::<T>::MintFailed)?;

          // burn from token
          T::CurrencyWrapr::burn_from(asset_id_from, &account_id, amount_from)
            .map_err(|_| Error::<T>::BurnFailed)?;

          // mint new tokens with fallback to restore token if it fails
          if T::CurrencyWrapr::mint_into(asset_id_to, &account_id, amount_to).is_err() {
            let revert = T::CurrencyWrapr::mint_into(asset_id_from, &account_id, amount_from);
            debug_assert!(revert.is_ok(), "withdrew funds previously; qed");
            return Err(Error::<T>::MintFailed.into());
          };

          // FIXME: we can probably remove this and only use the
          // event emitted by the Assets pallet
          // emit the burned event
          let request_id = T::Security::get_unique_id(account_id.clone());
          Self::deposit_event(Event::<T>::Traded(
            // fake request id
            request_id,
            account_id.clone(),
            asset_id_from,
            amount_from,
            asset_id_to,
            amount_to,
          ));
        }
        WithdrawConsequence::NoFunds => return Err(Error::<T>::NoFunds.into()),
        WithdrawConsequence::UnknownAsset => return Err(Error::<T>::UnknownAsset.into()),
        _ => return Err(Error::<T>::UnknownError.into()),
      };

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
      Self::deposit_event(Event::<T>::AccountChanged(new_account_id));

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
      Self::deposit_event(Event::<T>::StatusChanged(is_enabled));

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
        amount_to,
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
