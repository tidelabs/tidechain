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
    dispatch::{DispatchError, DispatchResult},
    inherent::Vec,
    pallet_prelude::*,
    traits::fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
    PalletId,
  };
  use frame_system::{ensure_root, pallet_prelude::*};
  #[cfg(feature = "std")]
  use sp_runtime::traits::AccountIdConversion;
  use sp_runtime::Permill;
  use sp_std::vec;
  use tidefi_primitives::{
    assets::Asset,
    pallet::{FeesExt, OracleExt, SecurityExt, SunriseExt},
    AssetId, Balance, CurrencyId, Fee, Hash, MarketPair, SlippageError, Swap, SwapConfirmation,
    SwapStatus, SwapType,
  };

  /// Oracle configuration
  #[pallet::config]
  pub trait Config:
    frame_system::Config + pallet_assets::Config<AssetId = AssetId, Balance = Balance>
  {
    /// Events
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

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

    /// The maximum number of supported market pairs
    #[pallet::constant]
    type SupportedMarketPairsLimit: Get<u32>;

    /// Fees traits
    type Fees: FeesExt<Self::AccountId, Self::BlockNumber>;

    /// Tidefi sunrise traits
    type Sunrise: SunriseExt<Self::AccountId, Self::BlockNumber>;

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

  /// Set of swap market pairs
  #[pallet::storage]
  #[pallet::getter(fn supported_market_pairs)]
  pub type SupportedMarketPairs<T: Config> =
    StorageValue<_, BoundedVec<MarketPair, T::SupportedMarketPairsLimit>, ValueQuery>;

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
    // List of supported market pairs
    pub market_pairs: Vec<MarketPair>,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        // Oracle is enabled by default
        enabled: true,
        // We use pallet account ID by default,
        // but should always be set in the genesis config.
        account: T::OraclePalletId::get().into_account_truncating(),
        market_makers: Vec::new(),
        market_pairs: Vec::new(),
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

      SupportedMarketPairs::<T>::put(BoundedVec::try_from(self.market_pairs.clone()).unwrap());
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
    /// Sudo added a market pair
    MarketPairAdded { market_pair: MarketPair },
    /// Sudo removed a market pair
    MarketPairRemoved { market_pair: MarketPair },
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
    /// Invalid swap request status.
    InvalidSwapStatus,
    /// Swap request assets must be different.
    SwapAssetsShouldBeDifferent,
    /// Invalid market maker request ID, includes an index in the SwapConfirmation list
    InvalidMarketMakerRequestId { index: u8 },
    /// Invalid market maker swap request status.
    InvalidMarketMakerSwapRequestStatus,
    /// Buyer and seller assets must match.
    BuySellAssetMismatch,
    /// Unknown Asset.
    UnknownAsset,
    /// Unable to calculate slippage
    SlippageOverflow,
    /// Arithmetic error
    ArithmeticError,
    /// Request contains offer that is less than swap lower bound
    OfferIsLessThanSwapLowerBound { index: u8 },
    /// Request contains offer that is greater than swap upper bound
    OfferIsGreaterThanSwapUpperBound { index: u8 },
    /// Request contains offer that is less than market maker swap lower bound
    OfferIsLessThanMarketMakerSwapLowerBound { index: u8 },
    /// Request contains offer that is greater than market maker swap upper bound
    OfferIsGreaterThanMarketMakerSwapUpperBound { index: u8 },
    /// Slippage validation blocks buying with low price
    NoLowerBoundForBuyingPrice,
    /// Slippage validation blocks selling with high price
    NoUpperBoundForSellingPrice,
    /// Swap is not created by a market maker
    NonMarketMakerSwap,
    /// Trader swap market pair is not supported
    MarketPairNotSupported,
    /// Market pair is already supported
    MarketPairAlreadySupported,
    /// The number of supported market pairs overflow.
    MarketPairOverflow,
    /// Market maker swap type is not limit
    MarketMakerSwapTypeIsNotLimit,
    /// Market Maker swap does not have enough funds left to sell
    MarketMakerSwapHasNotEnoughTokenLeftToSell,
    /// Trader swap does not have enough funds left to sell
    TraderSwapHasNotEnoughTokenLeftToSell,
    /// Trader and Market Maker swaps cannot oversell base asset
    CannotOversellBaseAsset,
    /// The fee cannot be greater than purchased amount
    FeeIsMoreThanPurchasedAmount,
    /// Seller does not hold enough funds to sell.
    NotHoldEnoughFundToSell,
    /// Cannot deposit funds to the buyer.
    CannotDepositToBuyer,
    /// Cannot deposit swap fees to the Fees account.
    CannotDepositSwapFee,
    /// Failed to update market maker swap
    UpdateMarketMakerSwapFailed,
    /// Delete trader's swap request from Swaps failed
    DeleteSwapFailed,
    /// Release unswapped funds failed
    ReleaseUnswappedFundsFailed,
    /// Update trader's swap request status in AccountSwaps failed
    UpdateAccountSwapRequestStatusFailed,
    /// Failed to update trader swap
    UpdateTraderSwapFailed,
    /// Transfer reserved trader selling tokens to market maker failed
    TransferTraderSwapAssetsToMarketMakerFailed,
    /// Transfer reserved trader swap fee to market maker failed
    TransferTraderSwapFeeFailed,
    /// Unable to register trade swap network fees.
    TraderSwapFeeRegistrationFailed,
    /// Transfer reserved market maker selling tokens to trader failed
    TransferMarketMakerSwapAssetsToTraderFailed,
    /// Transfer reserved market maker swap fee to trader failed
    TransferMarketMakerSwapFeeFailed,
    /// Unable to register market maker swap network fees.
    MarketMakerSwapFeeRegistrationFailed,
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
    #[pallet::call_index(0)]
    #[pallet::weight(<T as pallet::Config>::WeightInfo::confirm_swap())]
    pub fn confirm_swap(
      origin: OriginFor<T>,
      request_id: Hash,
      market_makers: Vec<SwapConfirmation>,
    ) -> DispatchResultWithPostInfo {
      // Make sure the oracle/chain is not paused
      Self::ensure_not_paused()?;

      // Make sure this is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(Some(sender) == Self::account_id(), Error::<T>::AccessDenied);

      // Make sure this trade exsits in Swaps
      let trade = Self::swaps(request_id).ok_or(Error::<T>::InvalidRequestId)?;

      // Make sure the trade status is either pending or partially filled
      ensure!(
        trade.status == SwapStatus::Pending || trade.status == SwapStatus::PartiallyFilled,
        Error::<T>::InvalidSwapStatus
      );

      // Make sure buy asset type is different from sell asset type
      ensure!(
        trade.token_to != trade.token_from,
        Error::<T>::SwapAssetsShouldBeDifferent
      );

      let mut trade_sold_amount: Balance = 0;
      let mut trade_bought_amount: Balance = 0;

      for (index, mm) in market_makers.iter().enumerate() {
        // Make sure request exsits in Swaps
        let market_maker_trade = Self::swaps(mm.request_id)
          .ok_or(Error::<T>::InvalidMarketMakerRequestId { index: index as u8 })?;

        let (trade_fee, market_maker_fee) = Self::validate_pair_of_swap_requests(
          index,
          &mm,
          &market_maker_trade,
          &trade,
          // Trade amount_from_filled is not updated untill all market maker swaps are processed.
          // trade_sold_amount represents how much has been filled so far
          trade.amount_from_filled + trade_sold_amount,
        )?;

        Self::do_swaps(
          &mm,
          &trade,
          trade_fee,
          &market_maker_trade,
          market_maker_fee,
        )?;

        trade_sold_amount += mm.amount_to_receive;
        trade_bought_amount += mm.amount_to_send;
      }

      // Make sure the `request_id` exist
      Swaps::<T>::try_mutate_exists(request_id, |trade_request| {
        if let Some(trade) = trade_request {
          trade.amount_from_filled += trade_sold_amount;
          trade.amount_to_filled += trade_bought_amount;

          Self::update_swap_and_requestor_account(trade, request_id)
            .map_err(|_| Error::<T>::UpdateTraderSwapFailed)?;

          // Emit trade event on chain
          Self::deposit_event(Event::<T>::SwapProcessed {
            request_id,
            initial_extrinsic_hash: trade.extrinsic_hash,
            status: trade.status.clone(),
            account_id: trade.account_id.clone(),
            currency_from: trade.token_from,
            currency_amount_from: trade_sold_amount,
            currency_to: trade.token_to,
            currency_amount_to: trade_bought_amount,
          });

          // Delete swap if it is completed or its type is Market
          if trade.status == SwapStatus::Completed || trade.swap_type == SwapType::Market {
            *trade_request = None;
          }

          Ok(())
        } else {
          Err(Error::<T>::InvalidRequestId)
        }
      })?;

      // Update last seen
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
    #[pallet::call_index(1)]
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
    #[pallet::call_index(2)]
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
    #[pallet::call_index(3)]
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

    /// Update assets values.
    ///
    /// - `value`: How many TDFY required for 1 Asset.
    ///
    /// The value should be formatted with TDFY decimals (12)
    ///
    /// Example:
    ///
    /// If the Bitcoin price is 0.001815 BTC (for 1 TDFY)
    /// You get 550.9641873278 TDFY for 1 BTC
    ///
    /// The value should be: `vec![(2, 550_964_187_327_800)]`
    ///
    /// ***
    ///
    /// If the ETH price is 0.03133 ETH (for 1 TDFY)
    /// You get 31.9182891796999 TDFY for 1 ETH
    ///
    /// The value sent should be: `vec![(4, 31_918_289_179_699)]`
    ///
    /// ***
    ///
    /// If the USDT price is 33.650000 USDT (for 1 TDFY)
    /// You get 0.029717682000 TDFY for 1 USDT
    ///
    /// The value sent should be: `vec![(4, 29_717_682_020)]`
    ///
    /// Weight: `O(1)`
    ///
    #[pallet::call_index(4)]
    #[pallet::weight(<T as pallet::Config>::WeightInfo::update_assets_value())]
    pub fn update_assets_value(
      origin: OriginFor<T>,
      value: Vec<(AssetId, Balance)>,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure this is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(Some(sender) == Self::account_id(), Error::<T>::AccessDenied);

      if !value.is_empty() {
        // 2. Update only if we provided at least one price
        T::Sunrise::register_exchange_rate(value)?;
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
    #[pallet::call_index(5)]
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
    #[pallet::call_index(6)]
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

    /// Add a new market pair to be supported
    ///
    /// - `market_pair`: Market pair
    ///
    /// Emits `MarketPairAdded` event when successful.
    ///
    #[pallet::call_index(7)]
    #[pallet::weight(<T as pallet::Config>::WeightInfo::add_market_pair())]
    pub fn add_market_pair(
      origin: OriginFor<T>,
      market_pair: MarketPair,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure this is from sudo user
      ensure_root(origin)?;

      // 2. Add the new market pair to the storage
      let mut supported_market_pairs = Self::supported_market_pairs();
      ensure!(
        !supported_market_pairs
          .iter()
          .any(|pair| *pair == market_pair),
        Error::<T>::MarketPairAlreadySupported
      );
      supported_market_pairs
        .try_push(market_pair.clone())
        .map_err(|_| Error::<T>::MarketPairOverflow)?;
      SupportedMarketPairs::<T>::put(supported_market_pairs);

      // 3. Emit event on chain
      Self::deposit_event(Event::<T>::MarketPairAdded { market_pair });

      // don't take tx fees on success
      Ok(Pays::No.into())
    }

    /// Remove a market pair from the supported list
    ///
    /// - `market_pair`: Market pair
    ///
    /// Emits `MarketPairRemoved` event when successful.
    ///
    #[pallet::call_index(8)]
    #[pallet::weight(<T as pallet::Config>::WeightInfo::remove_market_pair())]
    pub fn remove_market_pair(
      origin: OriginFor<T>,
      market_pair: MarketPair,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure this is from sudo user
      ensure_root(origin)?;

      // 2. Remove the market pair from the storage
      let mut supported_market_pairs = Self::supported_market_pairs();
      ensure!(
        supported_market_pairs.contains(&market_pair),
        Error::<T>::MarketPairNotSupported
      );
      supported_market_pairs.retain(|pair| *pair != market_pair);
      SupportedMarketPairs::<T>::put(supported_market_pairs);

      // 3. Emit event on chain
      Self::deposit_event(Event::<T>::MarketPairRemoved { market_pair });

      // don't take tx fees on success
      Ok(Pays::No.into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config> Pallet<T> {
    fn validate_pair_of_swap_requests(
      index: usize,
      mm: &SwapConfirmation,
      market_maker_trade: &Swap<T::AccountId, T::BlockNumber>,
      trade: &Swap<T::AccountId, T::BlockNumber>,
      trade_latest_from_filled: Balance,
    ) -> Result<(Balance, Balance), DispatchError> {
      //  Make sure request is either pending or partially filled
      ensure!(
        market_maker_trade.status == SwapStatus::Pending
          || market_maker_trade.status == SwapStatus::PartiallyFilled,
        Error::<T>::InvalidMarketMakerSwapRequestStatus
      );

      // Make sure swap buyer and seller currencies match
      ensure!(
        market_maker_trade.token_to == trade.token_from
          && market_maker_trade.token_from == trade.token_to,
        Error::<T>::BuySellAssetMismatch
      );

      // Make sure swap creator is a market maker
      MarketMakers::<T>::get(market_maker_trade.account_id.clone())
        .ok_or(Error::<T>::NonMarketMakerSwap)?;

      // Make sure market maker swap type must be limit
      ensure!(
        market_maker_trade.swap_type == SwapType::Limit,
        Error::<T>::MarketMakerSwapTypeIsNotLimit
      );

      // Decide offer asset amount based on market pair
      let market_pair = Self::get_swap_market_pair(trade)?;

      let offer_base_amount = if trade.token_from == market_pair.base_asset {
        mm.amount_to_receive
      } else {
        mm.amount_to_send
      };

      let offer_quote_amount = if trade.token_from == market_pair.base_asset {
        mm.amount_to_send
      } else {
        mm.amount_to_receive
      };

      trade
        .validate_slippage(
          &market_maker_trade,
          offer_base_amount,
          offer_quote_amount,
          &market_pair,
        )
        .map_err(|err| match err {
          SlippageError::UnknownAsset => Error::<T>::UnknownAsset,
          SlippageError::SlippageOverflow => Error::<T>::SlippageOverflow,
          SlippageError::ArithmeticError => Error::<T>::ArithmeticError,
          SlippageError::OfferIsLessThanSwapLowerBound => {
            Error::<T>::OfferIsLessThanSwapLowerBound { index: index as u8 }
          }
          SlippageError::OfferIsGreaterThanSwapUpperBound => {
            Error::<T>::OfferIsGreaterThanSwapUpperBound { index: index as u8 }
          }
          SlippageError::OfferIsLessThanMarketMakerSwapLowerBound => {
            Error::<T>::OfferIsLessThanMarketMakerSwapLowerBound { index: index as u8 }
          }
          SlippageError::OfferIsGreaterThanMarketMakerSwapUpperBound => {
            Error::<T>::OfferIsGreaterThanMarketMakerSwapUpperBound { index: index as u8 }
          }
          SlippageError::UnknownAssetInMarketPair => Error::<T>::MarketPairNotSupported,
          SlippageError::NoLowerBoundForBuyingPrice => Error::<T>::NoLowerBoundForBuyingPrice,
          SlippageError::NoUpperBoundForSellingPrice => Error::<T>::NoUpperBoundForSellingPrice,
        })?;

      let trader_fee = Self::validate_fund_transfers(
        &trade,
        mm.amount_to_receive,
        mm.amount_to_send,
        trade_latest_from_filled,
        true,
      )?;

      let market_maker_fee = Self::validate_fund_transfers(
        &market_maker_trade,
        mm.amount_to_send,
        mm.amount_to_receive,
        market_maker_trade.amount_from_filled,
        false,
      )?;

      Ok((trader_fee.fee, market_maker_fee.fee))
    }

    fn validate_fund_transfers(
      swap: &Swap<T::AccountId, T::BlockNumber>,
      amount_to_sell: Balance,
      amount_to_buy: Balance,
      amount_already_sold: Balance,
      is_trader_swap: bool,
    ) -> Result<Fee, Error<T>> {
      if is_trader_swap {
        swap
          .amount_from
          .checked_sub(amount_already_sold)
          .ok_or(Error::<T>::TraderSwapHasNotEnoughTokenLeftToSell)?
      } else {
        swap
          .amount_from
          .checked_sub(swap.amount_from_filled)
          .ok_or(Error::<T>::MarketMakerSwapHasNotEnoughTokenLeftToSell)?
      };

      // Make sure swap has enough funds before we can continue
      T::CurrencyTidefi::balance_on_hold(swap.token_from, &swap.account_id)
        .checked_sub(amount_to_sell)
        .ok_or(Error::<T>::NotHoldEnoughFundToSell)?;

      let amount_and_fee = T::Fees::calculate_swap_fees(
        swap.token_to,
        amount_to_buy,
        swap.swap_type.clone(),
        // The fee will be calculated base on user's role
        swap.is_market_maker,
      );
      let amount_to_receive = amount_to_buy
        .checked_sub(amount_and_fee.fee)
        .ok_or(Error::<T>::FeeIsMoreThanPurchasedAmount)?;

      // Make sure buyer can deposit the funds
      T::CurrencyTidefi::can_deposit(swap.token_to, &swap.account_id, amount_to_receive, false)
        .into_result()
        .map_err(|_| Error::<T>::CannotDepositToBuyer)?;

      // Make sure fees account can deposit trade fees
      T::CurrencyTidefi::can_deposit(
        swap.token_to,
        &T::Fees::account_id(),
        amount_and_fee.fee,
        false,
      )
      .into_result()
      .map_err(|_| Error::<T>::CannotDepositSwapFee)?;

      Ok(amount_and_fee)
    }

    fn get_swap_market_pair(
      trade: &Swap<T::AccountId, T::BlockNumber>,
    ) -> Result<MarketPair, Error<T>> {
      let markets = Self::supported_market_pairs();
      let maybe_market = markets
        .iter()
        .find(|m| {
          **m
            == MarketPair {
              base_asset: trade.token_from,
              quote_asset: trade.token_to,
            }
            || **m
              == MarketPair {
                base_asset: trade.token_to,
                quote_asset: trade.token_from,
              }
        })
        .ok_or(Error::<T>::MarketPairNotSupported)?;
      Ok(maybe_market.clone())
    }

    fn do_swaps(
      mm: &SwapConfirmation,
      trade: &Swap<T::AccountId, T::BlockNumber>,
      trade_fee: Balance,
      market_maker_trade: &Swap<T::AccountId, T::BlockNumber>,
      market_maker_fee: Balance,
    ) -> DispatchResult {
      match Self::transfer_swap_funds_and_fees(
        mm,
        trade,
        trade_fee,
        market_maker_trade,
        market_maker_fee,
      ) {
        Ok(_) => {
          Swaps::<T>::try_mutate_exists(mm.request_id, |mm_trade_request| {
            if let Some(market_maker_trade) = mm_trade_request {
              market_maker_trade.amount_from_filled = market_maker_trade
                .amount_from_filled
                .checked_add(mm.amount_to_send)
                .ok_or(Error::<T>::ArithmeticError)?;

              market_maker_trade.amount_to_filled = market_maker_trade
                .amount_to_filled
                .checked_add(mm.amount_to_receive)
                .ok_or(Error::<T>::ArithmeticError)?;

              Self::update_swap_and_requestor_account(market_maker_trade, mm.request_id)
                .map_err(|_| Error::<T>::UpdateMarketMakerSwapFailed)?;

              // Emit market maker trade event on chain
              Self::deposit_event(Event::<T>::SwapProcessed {
                request_id: mm.request_id,
                initial_extrinsic_hash: market_maker_trade.extrinsic_hash,
                status: market_maker_trade.status.clone(),
                account_id: market_maker_trade.account_id.clone(),
                currency_from: market_maker_trade.token_from,
                currency_amount_from: mm.amount_to_send,
                currency_to: market_maker_trade.token_to,
                currency_amount_to: mm.amount_to_receive,
              });

              // Delete swap if it is completed or its type is Market
              if market_maker_trade.status == SwapStatus::Completed
                || market_maker_trade.swap_type == SwapType::Market
              {
                *mm_trade_request = None;
              }

              Ok::<(), Error<T>>(())
            } else {
              Err(Error::<T>::InvalidRequestId.into())
            }
          })?;

          Ok(())
        }
        Err(e) => Err(e.into()),
      }
    }

    fn transfer_swap_funds_and_fees(
      mm: &SwapConfirmation,
      trade: &Swap<T::AccountId, T::BlockNumber>,
      trade_fee: Balance,
      market_maker_trade: &Swap<T::AccountId, T::BlockNumber>,
      market_maker_fee: Balance,
    ) -> Result<(), Error<T>> {
      // Transfer funds from market maker to the trader
      T::CurrencyTidefi::transfer_held(
        market_maker_trade.token_from,
        &market_maker_trade.account_id,
        &trade.account_id,
        mm.amount_to_send.saturating_sub(trade_fee),
        false,
        false,
      )
      .map_err(|_| Error::<T>::TransferMarketMakerSwapAssetsToTraderFailed)?;

      // Transfer market maker fee from market maker to fees account
      T::CurrencyTidefi::transfer_held(
        market_maker_trade.token_from,
        &market_maker_trade.account_id,
        &T::Fees::account_id(),
        trade_fee,
        false,
        false,
      )
      .map_err(|_| Error::<T>::TransferTraderSwapFeeFailed)?;

      // Register a new trading fees associated with the trader account.
      T::Fees::register_swap_fees(
        trade.account_id.clone(),
        trade.token_to,
        mm.amount_to_send,
        trade.swap_type.clone(),
        trade.is_market_maker,
      )
      .map_err(|_| Error::<T>::TraderSwapFeeRegistrationFailed)?;

      // Transfer funds from the trader to market maker
      T::CurrencyTidefi::transfer_held(
        trade.token_from,
        &trade.account_id,
        &market_maker_trade.account_id,
        mm.amount_to_receive.saturating_sub(market_maker_fee),
        false,
        false,
      )
      .map_err(|_| Error::<T>::TransferTraderSwapAssetsToMarketMakerFailed)?;

      // Transfer trade fee from the trader to fees account
      T::CurrencyTidefi::transfer_held(
        trade.token_from,
        &trade.account_id,
        &T::Fees::account_id(),
        market_maker_fee,
        false,
        false,
      )
      .map_err(|_| Error::<T>::TransferMarketMakerSwapFeeFailed)?;

      // Register a new trading fees associated with the market maker
      T::Fees::register_swap_fees(
        market_maker_trade.account_id.clone(),
        market_maker_trade.token_to,
        mm.amount_to_receive,
        market_maker_trade.swap_type.clone(),
        market_maker_trade.is_market_maker,
      )
      .map_err(|_| Error::<T>::MarketMakerSwapFeeRegistrationFailed)?;

      Ok(())
    }

    fn update_swap_and_requestor_account(
      swap: &mut Swap<T::AccountId, T::BlockNumber>,
      request_id: Hash,
    ) -> Result<(), DispatchError> {
      if swap.amount_from_filled == swap.amount_from {
        swap.status = SwapStatus::Completed;
      } else {
        swap.status = SwapStatus::PartiallyFilled;
      }

      if swap.status == SwapStatus::Completed || swap.swap_type == SwapType::Market {
        // Delete the swap from AccountSwaps, then release on hold funds
        Self::try_delete_account_swap(&swap.account_id, request_id)
          .map_err(|_| Error::<T>::DeleteSwapFailed)?;
        Self::swap_release_funds(swap).map_err(|_| Error::<T>::ReleaseUnswappedFundsFailed)?;
      } else {
        // Update the swap status in AccountSwaps
        Self::try_update_account_swap_status(&swap.account_id, request_id, swap.status.clone())
          .map_err(|_| Error::<T>::UpdateAccountSwapRequestStatusFailed)?;
      }

      Ok(())
    }

    fn swap_release_funds(swap: &Swap<T::AccountId, T::BlockNumber>) -> Result<(), DispatchError> {
      let amount_to_release = swap
        .amount_from //swap_amount_freezed
        // reduce filled amount
        .checked_sub(swap.amount_from_filled)
        .ok_or(Error::<T>::ArithmeticError)?;

      T::CurrencyTidefi::release(swap.token_from, &swap.account_id, amount_to_release, true)?;

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

    fn set_initial_market_pairs() {
      if Self::supported_market_pairs().is_empty() {
        let initial_supported_market_pairs: BoundedVec<MarketPair, T::SupportedMarketPairsLimit> =
          BoundedVec::try_from(vec![
            // ATH_USDC
            MarketPair {
              base_asset: Asset::AllTimeHigh.currency_id(),
              quote_asset: Asset::USDCoin.currency_id(),
            },
            // BTC_USDC
            MarketPair {
              base_asset: Asset::Bitcoin.currency_id(),
              quote_asset: Asset::USDCoin.currency_id(),
            },
            // ETH_USDC
            MarketPair {
              base_asset: Asset::Ethereum.currency_id(),
              quote_asset: Asset::USDCoin.currency_id(),
            },
            // TDFY_BTC
            MarketPair {
              base_asset: Asset::Tdfy.currency_id(),
              quote_asset: Asset::Bitcoin.currency_id(),
            },
            // TDFY_ETH
            MarketPair {
              base_asset: Asset::Tdfy.currency_id(),
              quote_asset: Asset::Ethereum.currency_id(),
            },
            // TDFY_USDC
            MarketPair {
              base_asset: Asset::Tdfy.currency_id(),
              quote_asset: Asset::USDCoin.currency_id(),
            },
          ])
          .unwrap();

        SupportedMarketPairs::<T>::put(initial_supported_market_pairs.clone());
      }
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

      // Freeze selling asset amount as fee is going to be deducted from the received asset amount
      T::CurrencyTidefi::hold(asset_id_from, &account_id, amount_from)?;

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

          // release unswapped funds
          Self::swap_release_funds(swap_intent)
            .map_err(|_| Error::<T>::ReleaseUnswappedFundsFailed)?;

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

  // hooks
  #[pallet::hooks]
  impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
    fn on_initialize(_now: T::BlockNumber) -> Weight {
      Self::set_initial_market_pairs();
      T::DbWeight::get().reads_writes(1 as u64, 1 as u64)
    }
  }
}
