// Copyright 2021-2022 Semantic Network Ltd.
// This file is part of Tidechain.

// Tidechain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tidechain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tidechain.  If not, see <http://www.gnu.org/licenses/>.

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
  use sp_runtime::Permill;
  use sp_std::vec;
  use tidefi_primitives::{
    assets::USDT,
    pallet::{FeesExt, OracleExt, SecurityExt},
    AssetId, Balance, CurrencyId, Hash, OracleImAlive, Swap, SwapConfirmation, SwapStatus,
    SwapType,
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

    /// The maximum number of active swaps per account id
    #[pallet::constant]
    type SwapLimitByAccount: Get<u32>;

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

  /// Oracle last seen
  #[pallet::storage]
  #[pallet::getter(fn last_seen)]
  pub(super) type LastSeen<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

  /// Oracle Account ID
  #[pallet::storage]
  #[pallet::getter(fn account_id)]
  pub type OracleAccountId<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

  /// Mapping of pending Swaps
  #[pallet::storage]
  #[pallet::getter(fn swaps)]
  pub type Swaps<T: Config> =
    StorageMap<_, Blake2_128Concat, Hash, Swap<T::AccountId, T::BlockNumber>>;

  /// Mapping of pending Swaps by AccountId
  #[pallet::storage]
  #[pallet::getter(fn account_swaps)]
  pub type AccountSwaps<T: Config> = CountedStorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<(Hash, SwapStatus), T::SwapLimitByAccount>,
  >;

  /// Set of active market makers
  #[pallet::storage]
  #[pallet::getter(fn market_makers)]
  pub type MarketMakers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool>;

  /// Genesis configuration
  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    /// Oracle status
    pub enabled: bool,
    /// Oracle Account ID. Multisig is supported.
    /// This account will be able to confirm trades on-chain.
    pub account: T::AccountId,
    // List of active market makers
    pub market_makers: Vec<T::AccountId>,
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
        market_makers: Vec::new(),
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      OracleStatus::<T>::put(self.enabled);
      OracleAccountId::<T>::put(self.account.clone());

      for account_id in self.market_makers.clone() {
        MarketMakers::<T>::insert(account_id, true);
      }
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Oracle status changed
    StatusChanged { is_enabled: bool },
    /// Oracle account changed
    AccountChanged { account_id: T::AccountId },
    /// Oracle added a market maker
    MarketMakerAdded { account_id: T::AccountId },
    /// Oracle removed a market maker
    MarketMakerRemoved { account_id: T::AccountId },
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
    /// Invalid market maker request ID, includes an index in the SwapConfirmation list
    InvalidMarketMakerRequestId { index: u8 },
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
    /// Market maker overflow
    MarketMakerOverflow,
    /// Market Makers do not have enough funds
    MarketMakerNoFunds,
    /// Market Makers cannot deposit source funds of the trade
    MarketMakerCantDeposit,
    /// Swaps cap reached for this account id
    SwapOverflow,
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

            for (index, mm) in market_makers.iter().enumerate() {
              let mm_trade_request = Swaps::<T>::try_get(mm.request_id)
                .map_err(|_| Error::<T>::InvalidMarketMakerRequestId { index: index as u8 })?;

              // validate user slippage tolerance
              let pay_per_token = trade.amount_from as f64 / trade.amount_to as f64;
              let pay_per_token_offered = mm.amount_to_receive as f64 / mm.amount_to_send as f64;
              let allowed_slippage = trade.slippage.deconstruct() as f64 / 1_000_000_f64;

              // limit order can match with smaller price
              if trade.swap_type != SwapType::Limit {
                let minimum_per_token = pay_per_token - (allowed_slippage * pay_per_token);
                ensure!(minimum_per_token <= pay_per_token_offered, Error::Overflow);
              }

              let maximum_per_token = pay_per_token + (allowed_slippage * pay_per_token);
              ensure!(maximum_per_token >= pay_per_token_offered, Error::Overflow);

              // validate mm slippage tolerance
              let pay_per_token =
                mm_trade_request.amount_from as f64 / mm_trade_request.amount_to as f64;
              let pay_per_token_offered = mm.amount_to_send as f64 / mm.amount_to_receive as f64;
              let allowed_slippage = mm_trade_request.slippage.deconstruct() as f64 / 1_000_000_f64;

              // limit order can match with smaller price
              if mm_trade_request.swap_type != SwapType::Limit {
                let minimum_per_token = pay_per_token - (allowed_slippage * pay_per_token);
                ensure!(
                  minimum_per_token <= pay_per_token_offered,
                  Error::MarketMakerOverflow
                );
              }

              let maximum_per_token = pay_per_token + (allowed_slippage * pay_per_token);
              ensure!(
                maximum_per_token >= pay_per_token_offered,
                Error::MarketMakerOverflow
              );

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

            ensure!(
              trade.amount_from_filled <= trade.amount_from,
              Error::Overflow
            );

            if trade.amount_from_filled == trade.amount_from {
              trade.status = SwapStatus::Completed;
            } else {
              trade.status = SwapStatus::PartiallyFilled;
            }

            // 10. Make sure the requester can deposit the new asset before initializing trade process
            T::CurrencyTidefi::can_deposit(trade.token_to, &trade.account_id, total_to)
              .into_result()
              .map_err(|_| Error::<T>::BurnFailed)?;

            for (index, mm) in market_makers.iter().enumerate() {
              Swaps::<T>::try_mutate_exists(mm.request_id, |mm_trade_request| {
                match mm_trade_request {
                  None => Err(Error::<T>::InvalidMarketMakerRequestId { index: index as u8 }),
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
                    let available_funds = market_maker_trade_intent.amount_from
                      - market_maker_trade_intent.amount_from_filled;

                    if available_funds
                      .saturating_add(market_maker_trade_intent.slippage * available_funds)
                      < mm.amount_to_send
                    {
                      return Err(Error::<T>::InvalidMarketMakerRequest);
                    }

                    market_maker_trade_intent.amount_from_filled += mm.amount_to_send;
                    market_maker_trade_intent.amount_to_filled += mm.amount_to_receive;

                    if market_maker_trade_intent.amount_from_filled
                      == market_maker_trade_intent.amount_from
                    {
                      // completed fill
                      market_maker_trade_intent.status = SwapStatus::Completed;
                    } else {
                      market_maker_trade_intent.status = SwapStatus::PartiallyFilled;
                    }

                    // 11. d) Transfer funds from the requester to the market makers
                    let amount_and_fee = T::Fees::calculate_swap_fees(
                      trade.token_from,
                      mm.amount_to_receive,
                      trade.swap_type.clone(),
                      trade.is_market_maker,
                    );

                    if T::CurrencyTidefi::transfer_held(
                      trade.token_from,
                      &trade.account_id,
                      &market_maker_trade_intent.account_id,
                      mm.amount_to_receive,
                      false,
                      false,
                    )
                    .is_err()
                    {
                      // FIXME: Add rollback
                    }

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
                      trade.swap_type.clone(),
                      trade.is_market_maker,
                    )
                    .map_err(|_| Error::<T>::FeesFailed)?;

                    // 12. a) Transfer funds from the market makers to the account
                    let amount_and_fee = T::Fees::calculate_swap_fees(
                      trade.token_to,
                      mm.amount_to_send,
                      market_maker_trade_intent.swap_type.clone(),
                      market_maker_trade_intent.is_market_maker,
                    );

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
                      market_maker_trade_intent.swap_type.clone(),
                      market_maker_trade_intent.is_market_maker,
                    )
                    .map_err(|_| Error::<T>::FeesFailed)?;

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

                    // 14. Delete the intent if it's completed or if it's a market order

                    // release order if its within slippage values
                    if market_maker_trade_intent.status == SwapStatus::Completed
                      || market_maker_trade_intent.swap_type == SwapType::Market
                    {
                      Self::try_delete_account_swap(
                        &market_maker_trade_intent.account_id,
                        mm.request_id,
                      )
                      .map_err(|_| Error::<T>::UnknownError)?;
                      Self::swap_release_funds(market_maker_trade_intent)
                        .map_err(|_| Error::<T>::ReleaseFailed)?;
                      *mm_trade_request = None;
                    } else {
                      Self::try_update_account_swap_status(
                        &market_maker_trade_intent.account_id,
                        mm.request_id,
                        market_maker_trade_intent.status.clone(),
                      )
                      .map_err(|_| Error::<T>::UnknownError)?;
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

            // 16. close the trade if it's complete or is a market order
            if trade.status == SwapStatus::Completed || trade.swap_type == SwapType::Market {
              Self::try_delete_account_swap(&trade.account_id, request_id)
                .map_err(|_| Error::<T>::UnknownError)?;
              Self::swap_release_funds(trade).map_err(|_| Error::<T>::ReleaseFailed)?;

              *trade_request = None;
            } else {
              Self::try_update_account_swap_status(
                &trade.account_id,
                request_id,
                trade.status.clone(),
              )
              .map_err(|_| Error::<T>::UnknownError)?;
            }
          }
        }

        Ok(())
      })?;

      // 15. Update last seen
      LastSeen::<T>::put(T::Security::get_current_block_count());

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

      // 5. Update last seen
      LastSeen::<T>::put(T::Security::get_current_block_count());

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

      // 4. Update last seen
      LastSeen::<T>::put(T::Security::get_current_block_count());

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

      // 4. Update last seen
      LastSeen::<T>::put(T::Security::get_current_block_count());

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
    #[pallet::weight(<T as pallet::Config>::WeightInfo::im_alive())]
    pub fn im_alive(origin: OriginFor<T>, im_alive: OracleImAlive) -> DispatchResultWithPostInfo {
      // 1. Make sure this is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(Some(sender) == Self::account_id(), Error::<T>::AccessDenied);

      // 2. Build final price vector
      let mut all_prices = Vec::new();

      for (currency_id, price) in im_alive.usdt_value {
        all_prices.push((currency_id, CurrencyId::Wrapped(USDT), price))
      }

      for (asset_id, price) in im_alive.tifi_value {
        all_prices.push((CurrencyId::Wrapped(asset_id), CurrencyId::Tifi, price))
      }

      if !all_prices.is_empty() {
        // Update only if we provided at least one price
        T::Fees::register_order_book_price(all_prices)?;
      }

      // 3. Update last seen
      LastSeen::<T>::put(T::Security::get_current_block_count());

      // don't take tx fees on success
      Ok(Pays::No.into())
    }

    /// Add market maker to the local storage
    ///
    /// Emits `StatusChanged` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::add_market_maker())]
    pub fn add_market_maker(
      origin: OriginFor<T>,
      account_id: T::AccountId,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure this is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(Some(sender) == Self::account_id(), Error::<T>::AccessDenied);

      // 2. Insert and make the account ID as a market maker (overwrite if already exist)
      MarketMakers::<T>::insert(account_id.clone(), true);

      // 3. Emit event on chain
      Self::deposit_event(Event::<T>::MarketMakerAdded { account_id });

      // 4. Update last seen
      LastSeen::<T>::put(T::Security::get_current_block_count());

      // don't take tx fees on success
      Ok(Pays::No.into())
    }

    /// Remove market maker to the local storage
    ///
    /// - `delete_orders`: Should we delete all existing swaps on chain for this user?
    ///
    /// Emits `StatusChanged` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::remove_market_maker())]
    pub fn remove_market_maker(
      origin: OriginFor<T>,
      account_id: T::AccountId,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure this is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(Some(sender) == Self::account_id(), Error::<T>::AccessDenied);

      // 2. Remove the market makers from the chain storage
      MarketMakers::<T>::remove(account_id.clone());

      // 3. Emit event on chain
      Self::deposit_event(Event::<T>::MarketMakerRemoved { account_id });

      // 4. Update last seen
      LastSeen::<T>::put(T::Security::get_current_block_count());

      // don't take tx fees on success
      Ok(Pays::No.into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config> Pallet<T> {
    fn swap_release_funds(trade: &Swap<T::AccountId, T::BlockNumber>) -> Result<(), DispatchError> {
      // real fees required
      let real_fees_amount = T::Fees::calculate_swap_fees(
        trade.token_from,
        trade.amount_from_filled,
        trade.swap_type.clone(),
        trade.is_market_maker,
      );
      let fees_with_slippage = T::Fees::calculate_swap_fees(
        trade.token_from,
        trade.amount_from,
        trade.swap_type.clone(),
        trade.is_market_maker,
      );

      let amount_to_release = trade
        .amount_from
        // reduce filled amount
        .saturating_sub(trade.amount_from_filled)
        // reduce un-needed locked fee
        .saturating_add(fees_with_slippage.fee.saturating_sub(real_fees_amount.fee));

      T::CurrencyTidefi::release(
        trade.token_from,
        &trade.account_id,
        amount_to_release,
        false,
      )
      .map_err(|_| Error::<T>::ReleaseFailed)?;

      Ok(())
    }

    fn ensure_not_paused() -> Result<(), DispatchError> {
      if Self::is_oracle_enabled() {
        Ok(())
      } else {
        Err(Error::<T>::OraclePaused.into())
      }
    }

    // delete the `AccountSwaps` storage where the tidefi
    // app subscribe to get latest trade status
    fn try_delete_account_swap(
      account_id: &T::AccountId,
      request_id: Hash,
    ) -> Result<(), DispatchError> {
      AccountSwaps::<T>::try_mutate_exists(account_id, |account_swaps| match account_swaps {
        Some(swaps) => {
          swaps.retain(|(swap_id, _)| *swap_id != request_id);
          Ok(())
        }
        None => Ok(()),
      })
    }

    fn try_update_account_swap_status(
      account_id: &T::AccountId,
      request_id: Hash,
      swap_status: SwapStatus,
    ) -> Result<(), DispatchError> {
      AccountSwaps::<T>::try_mutate_exists(account_id, |account_swaps| match account_swaps {
        Some(swaps) => match swaps
          .iter_mut()
          .find(|(swap_request, _)| *swap_request == request_id)
        {
          Some((_, status)) => {
            *status = swap_status;
            Ok(())
          }
          None => Ok(()),
        },
        None => Ok(()),
      })
    }
  }

  // implement the `OracleExt` functions
  impl<T: Config> OracleExt<T::AccountId, T::BlockNumber> for Pallet<T> {
    fn is_oracle_enabled() -> bool {
      // make sure the chain and the oracle pallet are enabled
      T::Security::is_chain_running() && Self::status()
    }

    fn is_market_maker(account_id: T::AccountId) -> Result<bool, DispatchError> {
      Ok(MarketMakers::<T>::get(account_id).unwrap_or(false))
    }

    fn add_new_swap_in_queue(
      account_id: T::AccountId,
      asset_id_from: CurrencyId,
      amount_from: Balance,
      asset_id_to: CurrencyId,
      amount_to: Balance,
      block_number: T::BlockNumber,
      extrinsic_hash: [u8; 32],
      is_market_maker: bool,
      swap_type: SwapType,
      slippage: Permill,
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
        is_market_maker,
        swap_type: swap_type.clone(),
        slippage,
      };

      // 6. Freeze asset
      let amount_and_fee =
        T::Fees::calculate_swap_fees(asset_id_from, amount_from, swap_type, is_market_maker);

      T::CurrencyTidefi::hold(
        asset_id_from,
        &account_id,
        amount_from.saturating_add(amount_and_fee.fee),
      )?;

      Swaps::<T>::insert(request_id, swap.clone());

      AccountSwaps::<T>::try_mutate(account_id, |account_swaps| match account_swaps {
        Some(swaps) => swaps
          .try_push((request_id, SwapStatus::Pending))
          .map_err(|_| Error::<T>::SwapOverflow),
        None => {
          let empty_bounded_vec: BoundedVec<(Hash, SwapStatus), T::SwapLimitByAccount> =
            vec![(request_id, SwapStatus::Pending)]
              .try_into()
              .map_err(|_| Error::<T>::UnknownError)?;

          *account_swaps = Some(empty_bounded_vec);
          Ok(())
        }
      })?;

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
          let amount_and_fee = T::Fees::calculate_swap_fees(
            swap_intent.token_from,
            swap_intent.amount_from,
            swap_intent.swap_type.clone(),
            swap_intent.is_market_maker,
          );
          let amount_to_release = swap_intent
            .amount_from
            // amount filled
            .saturating_sub(swap_intent.amount_from_filled);

          // FIXME: Should we refund the swap fee?
          // swap fee
          let real_amount_to_release = if swap_intent.amount_from_filled == 0 {
            amount_to_release.saturating_add(amount_and_fee.fee)
          } else {
            // real fees required
            let fees_amount_filled = T::Fees::calculate_swap_fees(
              swap_intent.token_from,
              swap_intent.amount_from_filled,
              swap_intent.swap_type.clone(),
              swap_intent.is_market_maker,
            );
            let fees_amount = T::Fees::calculate_swap_fees(
              swap_intent.token_from,
              swap_intent.amount_from,
              swap_intent.swap_type.clone(),
              swap_intent.is_market_maker,
            );

            amount_to_release.saturating_add(fees_amount.fee.saturating_sub(fees_amount_filled.fee))
          };

          T::CurrencyTidefi::release(
            swap_intent.token_from,
            &swap_intent.account_id,
            real_amount_to_release,
            false,
          )
          .map_err(|_| Error::<T>::ReleaseFailed)?;

          // delete the swap from the storage
          Self::try_delete_account_swap(&swap_intent.account_id, request_id)
            .map_err(|_| Error::<T>::UnknownError)?;

          *swap = None;

          Ok(())
        }
      })?;

      Ok(())
    }
  }
}
