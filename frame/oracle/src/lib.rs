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
    traits::fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
    PalletId,
  };
  use frame_system::pallet_prelude::*;
  #[cfg(feature = "std")]
  use sp_runtime::traits::AccountIdConversion;
  use tidefi_primitives::{
    pallet::{FeesExt, OracleExt, SecurityExt},
    AssetId, Balance, CurrencyId, Hash, Swap, SwapConfirmation, SwapStatus,
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

    /// Tidechain currency wrapper
    type CurrencyTidefi: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + InspectHold<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + MutateHold<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
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
  pub type OracleAccountId<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

  /// Mapping of pending Swaps
  #[pallet::storage]
  #[pallet::getter(fn trades)]
  pub type Swaps<T: Config> =
    StorageMap<_, Blake2_128Concat, Hash, Swap<T::AccountId, T::BlockNumber>>;

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
    /// Oracle processed the initial swap
    SwapProcessed {
      request_id: Hash,
      status: SwapStatus,
      account_id: T::AccountId,
      currency_from: CurrencyId,
      currency_amount_from: Balance,
      currency_to: CurrencyId,
      currency_amount_to: Balance,
      initial_extrinsic_hash: [u8; 32],
    },
    /// Oracle cancelled the initial swap and released the funds
    SwapCancelled { request_id: Hash },
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
    /// Unable to release funds.
    ReleaseFailed,
    /// Unable to take or calculate network fees.
    FeesFailed,
    /// Unknown Asset.
    UnknownAsset,
    /// No Funds available for this Asset Id.
    NoFunds,
    /// Swap overflow
    Overflow,
    /// Market Makers do not have enough funds
    MarketMakerNoFunds,
    /// Market Makers cannot deposit source funds of the trade
    MarketMakerCantDeposit,
    /// Unknown Error.
    UnknownError,
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Oracle have confirmation and confirm the trade.
    ///
    /// - `request_id`: Unique request ID.
    /// - `market_makers`: Vector of `SwapConfirmation` who represent the allocation of multiple source.
    ///
    /// Emits `SwapProcessed` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::confirm_swap())]
    pub fn confirm_swap(
      origin: OriginFor<T>,
      request_id: Hash,
      market_makers: Vec<SwapConfirmation>,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the oracle/chain is not paused
      Self::ensure_not_paused()?;

      // 2. Make sure this is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(Some(sender) == Self::account_id(), Error::<T>::AccessDenied);

      // 3. Make sure the `request_id` exist
      Swaps::<T>::try_mutate_exists(request_id, |trade_request| {
        match trade_request {
          None => {
            return Err(Error::<T>::InvalidRequestId);
          }
          Some(trade) => {
            // 5. Make sure the trade status is pending or partially filled
            if trade.status != SwapStatus::Pending && trade.status != SwapStatus::PartiallyFilled {
              return Err(Error::<T>::InvalidRequestStatus);
            }

            // 6. Calculate totals and all market makers
            let mut total_from: Balance = 0;
            let mut total_to: Balance = 0;

            for mm in market_makers.iter() {
              let mm_trade_request = Swaps::<T>::try_get(mm.request_id)
                .map_err(|_| Error::<T>::InvalidMarketMakerRequest)?;

              // make sure all the market markers have enough funds before we can continue
              T::CurrencyTidefi::balance_on_hold(trade.token_to, &mm_trade_request.account_id)
                .checked_sub(mm.amount_to_send)
                .ok_or(Error::<T>::MarketMakerNoFunds)?;

              // make sure the `account_id` can withdraw the funds
              T::CurrencyTidefi::balance_on_hold(trade.token_from, &trade.account_id)
                .checked_sub(mm.amount_to_receive)
                .ok_or(Error::<T>::NoFunds)?;

              // make sure we are allowed to send the funds
              T::CurrencyTidefi::can_deposit(
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

            // 7. a) Validate totals
            trade.amount_from_filled += total_from;
            trade.amount_to_filled += total_to;

            if total_from < trade.amount_from {
              trade.status = SwapStatus::PartiallyFilled;
              // trade overflow
              if trade.amount_from_filled > trade.amount_from {
                return Err(Error::Overflow);
              }
            }

            if trade.amount_from == trade.amount_from_filled {
              trade.status = SwapStatus::Completed;
            }

            // FIXME: Add slippage check for partial fill
            // the best would probably get the average value for 1 token, then do the check for 1:1 with provided slippage calculation
            // 7. b) Maximum of 10% slippage for the `amount_to`
            //if trade.amount_to_filled * 10 < trade.amount_to_filled * 9 || trade.amount_to_filled * 10 > trade.amount_to_filled * 11 {
            //  return Err(Error::Conflict);
            //}

            // 9. Calculate and transfer network fee
            // FIXME: Should we take a transfer fee on the FROM or the TO asset or both?
            let _trading_fees = T::Fees::calculate_swap_fees(trade.token_to, total_to);

            // 10. Make sure the requester can deposit the new asset before initializing trade process
            T::CurrencyTidefi::can_deposit(trade.token_to, &trade.account_id, total_to)
              .into_result()
              .map_err(|_| Error::<T>::BurnFailed)?;

            for mm in market_makers.iter() {
              Swaps::<T>::try_mutate_exists(mm.request_id, |mm_trade_request| {
                match mm_trade_request {
                  None => Err(Error::<T>::InvalidRequestId),
                  Some(market_maker_trade_intent) => {
                    // 11. a) Make sure the marketmaker trade request is still valid
                    if market_maker_trade_intent.status != SwapStatus::Pending
                      && market_maker_trade_intent.status != SwapStatus::PartiallyFilled
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
                      market_maker_trade_intent.status = SwapStatus::PartiallyFilled;
                    }

                    market_maker_trade_intent.amount_from_filled += mm.amount_to_send;
                    market_maker_trade_intent.amount_to_filled += mm.amount_to_receive;

                    if market_maker_trade_intent.amount_from_filled
                      == market_maker_trade_intent.amount_from
                    {
                      // completed fill
                      market_maker_trade_intent.status = SwapStatus::Completed;
                    }

                    // 11. d) Transfer funds from the requester to the market makers
                    let amount_and_fee =
                      T::Fees::calculate_swap_fees(trade.token_from, mm.amount_to_receive);
                    if T::CurrencyTidefi::transfer_held(
                      trade.token_from,
                      &trade.account_id,
                      &market_maker_trade_intent.account_id,
                      // deduce the fee from the amount
                      mm.amount_to_receive,
                      false,
                      false,
                    )
                    .is_err()
                    {
                      // FIXME: Add rollback
                    }

                    // 11. e) Requester pay fees of the transaction, but this is deducted
                    // from the MM final amount, so this is paid by the MM
                    if T::CurrencyTidefi::transfer_held(
                      trade.token_from,
                      &trade.account_id,
                      &T::Fees::account_id(),
                      amount_and_fee.fee,
                      false,
                      false,
                    )
                    .is_err()
                    {
                      // FIXME: Add rollback
                    }

                    // 11. f) Register a new trading fees associated with the account.
                    // A percentage of the network profits will be re-distributed to the account at the end of the era.
                    T::Fees::register_swap_fees(
                      trade.account_id.clone(),
                      trade.token_from,
                      mm.amount_to_receive,
                    );

                    // 12. a) Transfer funds from the market makers to the account
                    let amount_and_fee =
                      T::Fees::calculate_swap_fees(trade.token_to, mm.amount_to_send);

                    if T::CurrencyTidefi::transfer_held(
                      trade.token_to,
                      &market_maker_trade_intent.account_id,
                      &trade.account_id,
                      // deduce the fee from the amount
                      mm.amount_to_send,
                      false,
                      false,
                    )
                    .is_err()
                    {
                      // FIXME: Add rollback
                    }

                    // 12. b) Market makers pay fees of the transaction, but this is deducted
                    // from the requester final amount, so this is paid by the requester
                    if T::CurrencyTidefi::transfer_held(
                      trade.token_to,
                      &market_maker_trade_intent.account_id,
                      &T::Fees::account_id(),
                      amount_and_fee.fee,
                      false,
                      false,
                    )
                    .is_err()
                    {
                      // FIXME: Add rollback
                    }

                    // 12. c) Register a new trading fees associated with the account.
                    // A percentage of the network profits will be re-distributed to the account at the end of the era.
                    T::Fees::register_swap_fees(
                      market_maker_trade_intent.account_id.clone(),
                      trade.token_to,
                      mm.amount_to_send,
                    );

                    // 13. Emit market maker trade event on chain
                    Self::deposit_event(Event::<T>::SwapProcessed {
                      request_id: mm.request_id,
                      initial_extrinsic_hash: market_maker_trade_intent.extrinsic_hash,
                      status: market_maker_trade_intent.status.clone(),
                      account_id: market_maker_trade_intent.account_id.clone(),
                      currency_from: market_maker_trade_intent.token_from,
                      currency_amount_from: mm.amount_to_send,
                      currency_to: market_maker_trade_intent.token_to,
                      currency_amount_to: mm.amount_to_receive,
                    });

                    // 14. Delete the intent if it's completed
                    if market_maker_trade_intent.amount_from
                      == market_maker_trade_intent.amount_from_filled
                    {
                      *mm_trade_request = None;
                    }

                    Ok(())
                  }
                }
              })?;
            }

            // 15. Emit event on chain
            Self::deposit_event(Event::<T>::SwapProcessed {
              request_id,
              initial_extrinsic_hash: trade.extrinsic_hash,
              status: trade.status.clone(),
              account_id: trade.account_id.clone(),
              currency_from: trade.token_from,
              currency_amount_from: total_from,
              currency_to: trade.token_to,
              currency_amount_to: total_to,
            });

            // 16. close the trade if it's complete (we don't use the amount_to compare as there is a slippage to validate)
            if trade.amount_from == trade.amount_from_filled {
              // delete the trade request
              *trade_request = None;
            }
          }
        }

        Ok(())
      })?;

      // don't take tx fees on success
      Ok(Pays::No.into())
    }

    /// Oracle cancel a swap request and release remaining funds
    ///
    /// - `request_id`: Unique request ID.
    ///
    /// Emits `SwapCancelled` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::confirm_swap())]
    pub fn cancel_swap(origin: OriginFor<T>, request_id: Hash) -> DispatchResultWithPostInfo {
      // 1. Make sure the oracle/chain is not paused
      Self::ensure_not_paused()?;

      // 2. Make sure this is signed by `account_id`
      let sender = ensure_signed(origin)?;

      // 3. Remove swap from queue
      Self::remove_swap_from_queue(sender, request_id)?;

      // 4. Emit event on chain
      Self::deposit_event(Event::<T>::SwapCancelled { request_id });

      Ok(Pays::No.into())
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
      ensure!(Some(sender) == Self::account_id(), Error::<T>::AccessDenied);

      // 2. Update oracle account
      OracleAccountId::<T>::put(new_account_id.clone());

      // 3. Emit event on chain
      Self::deposit_event(Event::<T>::AccountChanged {
        account_id: new_account_id,
      });

      // don't take tx fees on success
      Ok(Pays::No.into())
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
      ensure!(Some(sender) == Self::account_id(), Error::<T>::AccessDenied);

      // 2. Update oracle status
      OracleStatus::<T>::set(is_enabled);

      // 3. Emit event on chain
      Self::deposit_event(Event::<T>::StatusChanged { is_enabled });

      // don't take tx fees on success
      Ok(Pays::No.into())
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

    fn add_new_swap_in_queue(
      account_id: T::AccountId,
      asset_id_from: CurrencyId,
      amount_from: Balance,
      asset_id_to: CurrencyId,
      amount_to: Balance,
      block_number: T::BlockNumber,
      extrinsic_hash: [u8; 32],
    ) -> Result<(Hash, Swap<T::AccountId, T::BlockNumber>), DispatchError> {
      let request_id = T::Security::get_unique_id(account_id.clone());
      let swap = Swap {
        account_id: account_id.clone(),
        token_from: asset_id_from,
        token_to: asset_id_to,
        amount_from,
        amount_from_filled: 0,
        amount_to,
        amount_to_filled: 0,
        status: SwapStatus::Pending,
        block_number,
        extrinsic_hash,
      };

      // 6. Freeze asset
      let amount_and_fee = T::Fees::calculate_swap_fees(asset_id_from, amount_from);
      T::CurrencyTidefi::hold(
        asset_id_from,
        &account_id,
        amount_from.saturating_add(amount_and_fee.fee),
      )?;

      Swaps::<T>::insert(request_id, swap.clone());
      Ok((request_id, swap))
    }

    fn remove_swap_from_queue(
      requester: T::AccountId,
      request_id: Hash,
    ) -> Result<(), DispatchError> {
      Swaps::<T>::try_mutate_exists(request_id, |swap| match swap {
        None => Err(Error::<T>::InvalidRequestId),
        Some(swap_intent) => {
          // allow oracle or the requester to cancel the swap
          ensure!(
            Some(requester.clone()) == Self::account_id() || swap_intent.account_id == requester,
            Error::<T>::AccessDenied
          );

          // release the remaining funds and the network fee
          let amount_and_fee =
            T::Fees::calculate_swap_fees(swap_intent.token_from, swap_intent.amount_from);
          let amount_to_release = swap_intent
            .amount_from
            .saturating_sub(swap_intent.amount_from_filled)
            .saturating_add(amount_and_fee.fee);

          T::CurrencyTidefi::release(
            swap_intent.token_from,
            &swap_intent.account_id,
            amount_to_release,
            false,
          )
          .map_err(|_| Error::<T>::ReleaseFailed)?;

          // delete the swap from the storage
          *swap = None;

          Ok(())
        }
      })?;

      Ok(())
    }
  }
}
