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

pub mod weights;
pub use weights::*;

mod migrations;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

pub(crate) const LOG_TARGET: &str = "tidefi::fees";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: crate::LOG_TARGET,
			concat!("[{:?}] 💸 ", $patter), T::Security::get_current_block_count() $(, $values)*
		)
	};
}

#[frame_support::pallet]
pub mod pallet {
  use super::*;
  use frame_support::{
    inherent::Vec,
    log,
    pallet_prelude::*,
    traits::{
      tokens::fungibles::{Inspect, Mutate, Transfer},
      StorageVersion, UnixTime,
    },
    PalletId,
  };
  use frame_system::pallet_prelude::*;
  use sp_arithmetic::traits::Zero;
  use sp_runtime::{
    traits::{AccountIdConversion, CheckedDiv, CheckedMul, Saturating},
    FixedPointNumber, FixedU128, Percent, Permill, SaturatedConversion,
  };
  use sp_std::{borrow::ToOwned, vec};
  use tidefi_primitives::{
    assets::{Asset, USDT},
    pallet::{FeesExt, SecurityExt, StakingExt},
    ActiveEraInfo, Balance, CurrencyId, EraIndex, Fee, SessionIndex, SunriseSwapPool, SwapType,
  };

  /// The current storage version.
  const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

  type BoundedAccountFees = BoundedVec<(CurrencyId, Fee), ConstU32<1_000>>;
  type BoundedSunrisePools = BoundedVec<SunriseSwapPool, ConstU32<6>>;

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config: frame_system::Config + pallet_security::Config {
    /// Events
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    /// Time used for computing era duration.
    ///
    /// It is guaranteed to start being called from the first `on_finalize`. Thus value at
    /// genesis is not used.
    type UnixTime: UnixTime;

    /// Pallet ID
    #[pallet::constant]
    type FeesPalletId: Get<PalletId>;

    /// Number of sessions per era
    #[pallet::constant]
    type SessionsPerEra: Get<SessionIndex>;

    /// Number of sessions to keep in archive
    #[pallet::constant]
    type SessionsArchive: Get<SessionIndex>;

    /// Number of block per session
    #[pallet::constant]
    type BlocksPerSession: Get<Self::BlockNumber>;

    /// Number of blocks to wait before allowing users to claim their sunrise rewards, after an era is completed.
    #[pallet::constant]
    type BlocksSunriseClaims: Get<Self::BlockNumber>;

    /// Retail user swap fee
    #[pallet::constant]
    type FeeAmount: Get<Permill>;

    /// Market maker market order fee
    #[pallet::constant]
    type MarketMakerFeeAmount: Get<Permill>;

    /// Market maker limit order fee
    #[pallet::constant]
    type MarketMakerLimitFeeAmount: Get<Permill>;

    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;

    /// Security traits
    type Security: SecurityExt<Self::AccountId, Self::BlockNumber>;

    /// Tidefi stake traits
    type Staking: StakingExt<Self::AccountId>;

    /// Tidechain currency wrapper
    type CurrencyTidefi: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

    /// The origin which may forcibly update the fee and distribution percentage
    type ForceOrigin: EnsureOrigin<Self::Origin>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  #[pallet::storage_version(STORAGE_VERSION)]
  pub struct Pallet<T>(_);

  /// The active era information, it holds index and start.
  ///
  /// The active era is the era being currently rewarded.
  #[pallet::storage]
  #[pallet::getter(fn active_era)]
  pub type ActiveEra<T: Config> = StorageValue<_, ActiveEraInfo<T::BlockNumber>>;

  /// The active sunrise tier availables.
  #[pallet::storage]
  #[pallet::getter(fn sunrise_pools)]
  pub type SunrisePools<T: Config> = StorageValue<_, BoundedSunrisePools, ValueQuery>;

  /// The current session of the era.
  #[pallet::storage]
  #[pallet::getter(fn current_session)]
  pub type CurrentSession<T: Config> = StorageValue<_, SessionIndex, ValueQuery>;

  /// The total fees for the era.
  /// If total hasn't been set or has been removed then 0 stake is returned.
  #[pallet::storage]
  #[pallet::getter(fn era_total_fees)]
  pub type EraTotalFees<T: Config> =
    StorageDoubleMap<_, Blake2_128Concat, EraIndex, Blake2_128Concat, CurrencyId, Fee, ValueQuery>;

  /// Map from all stored sessions.
  #[pallet::storage]
  #[pallet::getter(fn stored_sessions)]
  pub type StoredSessions<T: Config> = StorageMap<_, Blake2_128Concat, SessionIndex, ()>;

  /// Tdfy price of the orderbook reported by oracle every X minutes at the current market price.
  /// We keep in sync order book of USDT values for our sunrise pool.
  ///
  /// CurrencyId → USDT
  /// USDT → TDFY
  ///
  /// To get current TDFY USDT value;
  ///
  #[pallet::storage]
  #[pallet::getter(fn order_book_price)]
  pub type OrderBookPrice<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    CurrencyId,
    Blake2_128Concat,
    CurrencyId,
    FixedU128,
    ValueQuery,
  >;

  /// The total fees for the session.
  /// If total hasn't been set or has been removed then 0 stake is returned.
  #[pallet::storage]
  #[pallet::getter(fn session_total_fees)]
  pub type SessionTotalFees<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    SessionIndex,
    Blake2_128Concat,
    CurrencyId,
    Fee,
    ValueQuery,
  >;

  /// Account fees accumulated by eras
  #[pallet::storage]
  #[pallet::getter(fn account_fees)]
  pub type AccountFees<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    EraIndex,
    Blake2_128Concat,
    T::AccountId,
    BoundedAccountFees,
    ValueQuery,
  >;

  /// Account fees for current era
  #[pallet::storage]
  #[pallet::getter(fn sunrise_rewards)]
  pub type SunriseRewards<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    Blake2_128Concat,
    EraIndex,
    Balance,
    ValueQuery,
  >;

  /// Genesis configuration
  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    pub sunrise_swap_pools: Vec<SunriseSwapPool>,
    pub phantom: PhantomData<T>,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        phantom: PhantomData,
        sunrise_swap_pools: Vec::new(),
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      let bounded_sunrise_pool: BoundedSunrisePools =
        self.sunrise_swap_pools.clone().try_into().unwrap();
      SunrisePools::<T>::put(bounded_sunrise_pool);
      CurrentSession::<T>::put(1);
      ActiveEra::<T>::put(ActiveEraInfo::<T::BlockNumber> {
        index: 1,
        // Set new active era start in next `on_finalize`. To guarantee usage of `Time`
        start_block: None,
        start_session_index: None,
        last_session_block: None,
        start: None,
      });

      // Create Fee account
      let account_id = <Pallet<T>>::account_id();
      let min = T::CurrencyTidefi::minimum_balance(CurrencyId::Tdfy);
      if T::CurrencyTidefi::reducible_balance(CurrencyId::Tdfy, &account_id, false) < min {
        if let Err(err) = T::CurrencyTidefi::mint_into(CurrencyId::Tdfy, &account_id, min) {
          log!(
            error,
            "Unable to mint fee pallet minimum balance: {:?}",
            err
          );
        }
      }
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    FeesPercentageUpdated(Percent),
    SessionEnded {
      era_index: EraIndex,
      session_index: SessionIndex,
      session_fees_by_currency: Vec<(CurrencyId, Balance)>,
    },
    EraStarted {
      era_index: EraIndex,
    },
    EraEnded {
      era_index: EraIndex,
    },
    SunriseRewarded {
      era_index: EraIndex,
      pool_id: u8,
      account_id: T::AccountId,
      reward: Balance,
    },
    SunriseClaimed {
      era_index: EraIndex,
      account_id: T::AccountId,
      reward: Balance,
    },
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Invalid sunrise pool
    InvalidSunrisePool,
    /// There is no rewards available for this account on this era
    NoRewardsAvailable,
    /// Invalid era
    InvalidEra,
    /// Invalid asset
    InvalidAsset,
    /// There is no active Era
    NoActiveEra,
    /// There is no active Era start block
    NoActiveEraStartBlock,
    /// Era is not ready to be claimed yet, try again later
    EraNotReady,
    /// Account fees overflow
    AccountFeeOverflow,
    /// Balance overflow
    BalanceOverflow,
    /// Invalid USDT value in the order book
    InvalidUsdtValue,
    /// Invalid TDFY value in the order book
    InvalidTdfyValue,
  }

  // hooks
  #[pallet::hooks]
  impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
      migrations::migrate_to_v1::<T, Self>()
    }

    fn on_initialize(_now: T::BlockNumber) -> Weight {
      // just return the weight of the on_finalize.
      T::DbWeight::get().reads(2)
    }

    fn on_finalize(_current_block: T::BlockNumber) {
      if let Some(mut active_era) = Self::active_era() {
        let real_block = T::Security::get_current_block_count();
        match active_era.start_block {
          Some(start_block) => {
            // determine when the session
            let session_start_block = match active_era.last_session_block {
              Some(last_session_block) => last_session_block,
              None => start_block,
            };
            let expected_end_block_for_session =
              session_start_block.saturating_add(T::BlocksPerSession::get());

            // end of session
            if real_block == expected_end_block_for_session {
              let current_session = CurrentSession::<T>::get();

              let expected_end_session_for_era = match active_era.start_session_index {
                Some(start_session_index) => start_session_index,
                None => current_session,
              }
              .saturating_add(T::SessionsPerEra::get());

              log!(
                debug,
                "Fees compound session #{} started in block #{:?}, and is now expired.",
                current_session,
                start_block
              );

              // get current session total trade / currency
              let session_fees_by_currency: Vec<(CurrencyId, Balance)> =
                SessionTotalFees::<T>::iter_prefix(current_session)
                  .map(|(currency_id, fee)| (currency_id, fee.fee))
                  .collect();

              // notify the staking pallet that we are done with this session
              // the compute can be done for all stakers
              if let Err(err) =
                T::Staking::on_session_end(current_session, session_fees_by_currency.clone())
              {
                log!(error, "Can't notify staking pallet {:?}", err);
              }

              // Emit end of session event on chain
              Self::deposit_event(Event::<T>::SessionEnded {
                era_index: active_era.index,
                session_index: current_session,
                session_fees_by_currency,
              });

              // increment our session
              let new_session = current_session.saturating_add(1_u64);
              CurrentSession::<T>::put(new_session);
              StoredSessions::<T>::insert(current_session, ());
              // record the session change for the era
              active_era.last_session_block = Some(real_block);

              if current_session == expected_end_session_for_era {
                Self::deposit_event(Event::<T>::EraEnded {
                  era_index: active_era.index,
                });
                // increment the era index
                active_era.index = active_era.index.saturating_add(1);
                // reset the era values
                active_era.last_session_block = None;
                active_era.start_block = None;
                active_era.start_session_index = None;
                active_era.start = None;
              }

              // update active era
              ActiveEra::<T>::put(active_era);

              // drain old sessions
              Self::drain_old_sessions();
            }
          }
          None => {
            let now_as_millis_u64 = T::UnixTime::now().as_millis().saturated_into::<u64>();
            active_era.start = Some(now_as_millis_u64);
            active_era.start_block = Some(real_block);
            active_era.start_session_index = Some(CurrentSession::<T>::get());
            // This write only ever happens once, we don't include it in the weight in
            // general
            Self::deposit_event(Event::<T>::EraStarted {
              era_index: active_era.index,
            });
            ActiveEra::<T>::put(active_era);
            log!(trace, "Initializing in block #{:?}", real_block);
          }
        }
      }
      // `on_finalize` weight is tracked in `on_initialize`
    }
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Claim available sunrise rewards
    ///
    /// - `era_index`: Era to claim rewards
    ///
    /// Emits `SunriseRewardsClaimed` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::claim_sunrise_rewards())]
    pub fn claim_sunrise_rewards(
      origin: OriginFor<T>,
      era_index: EraIndex,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the transaction is signed
      let account_id = ensure_signed(origin)?;

      // 2. Make sure the era Index provided is ready to be claimed
      let current_era = Self::active_era().ok_or(Error::<T>::NoActiveEra)?;
      let starting_block = current_era
        .start_block
        .ok_or(Error::<T>::NoActiveEraStartBlock)?;
      let current_block = T::Security::get_current_block_count();

      // Unable to claim current Era
      if era_index >= current_era.index {
        return Err(Error::<T>::InvalidEra.into());
      }

      // Unable to claim previous era if the `T::BlocksSunriseClaims` cooldown isnt completed
      if era_index == current_era.index.saturating_sub(1)
        && starting_block.saturating_add(T::BlocksSunriseClaims::get()) > current_block
      {
        return Err(Error::<T>::EraNotReady.into());
      }

      // 3. Claim rewards
      Self::try_claim_sunrise_rewards(&account_id, era_index)?;

      // Don't take tx fees on success
      Ok(Pays::No.into())
    }
  }

  impl<T: Config> Pallet<T> {
    // Initialize sunrise reward
    pub fn try_claim_sunrise_rewards(
      who: &T::AccountId,
      era_index: EraIndex,
    ) -> Result<(), DispatchError> {
      SunriseRewards::<T>::try_mutate_exists(who, era_index, |found_reward| match found_reward {
        Some(reward) => {
          if *reward == 0 {
            return Err(Error::<T>::NoRewardsAvailable.into());
          }

          // transfer funds
          T::CurrencyTidefi::transfer(CurrencyId::Tdfy, &Self::account_id(), who, *reward, true)?;

          // emit event
          Self::deposit_event(Event::<T>::SunriseClaimed {
            era_index,
            account_id: who.clone(),
            reward: *reward,
          });

          // delete storage
          *found_reward = None;
          Ok(())
        }
        None => Err(Error::<T>::NoRewardsAvailable.into()),
      })
    }

    // Delete all session where the index < current_session - T::SessionsArchive
    pub(crate) fn drain_old_sessions() {
      let current_session = CurrentSession::<T>::get();
      for (session, _) in StoredSessions::<T>::iter() {
        if session < current_session.saturating_sub(T::SessionsArchive::get()) {
          // delete the session
          let _ = SessionTotalFees::<T>::clear_prefix(session, u32::MAX, None);
          StoredSessions::<T>::remove(session);
        }
      }
    }

    // Based on the price provided by Oracle, try to convert the usdt amount to the asset balance
    pub(crate) fn try_get_value_from_usdt(
      currency_id: CurrencyId,
      usdt_amount: FixedU128,
    ) -> Result<Balance, DispatchError> {
      let asset_from: Asset = currency_id
        .try_into()
        .map_err(|_| Error::<T>::InvalidAsset)?;

      // get order book price
      let order_book_price = if currency_id == CurrencyId::Wrapped(USDT) {
        FixedU128::from(1)
      } else {
        Self::order_book_price(currency_id, CurrencyId::Wrapped(USDT))
      };

      if order_book_price.is_zero() || usdt_amount.is_zero() {
        return Ok(0);
      }

      let currency_wanted = (usdt_amount * order_book_price)
        .checked_div(&FixedU128::from(
          10_u128.pow(Asset::Tether.exponent() as u32),
        ))
        .ok_or(Error::<T>::BalanceOverflow)?;

      Ok(
        currency_wanted
          .checked_mul(&FixedU128::from(10_u128.pow(asset_from.exponent() as u32)))
          .ok_or(Error::<T>::InvalidUsdtValue)?
          .into_inner()
          .checked_div(FixedU128::DIV)
          .ok_or(Error::<T>::BalanceOverflow)?,
      )
    }

    // Based on the value provided by Oracle, try to convert the asset balance to USDT balance
    pub(crate) fn try_get_usdt_value(
      currency_id: CurrencyId,
      amount: FixedU128,
    ) -> Result<Balance, DispatchError> {
      let asset_from: Asset = currency_id
        .try_into()
        .map_err(|_| Error::<T>::InvalidAsset)?;

      let order_book_price = if currency_id == CurrencyId::Wrapped(USDT) {
        FixedU128::from(1)
      } else {
        Self::order_book_price(currency_id, CurrencyId::Wrapped(USDT))
      };

      if order_book_price.is_zero() {
        return Ok(0);
      }

      let currency_wanted = amount
        .checked_div(&order_book_price)
        .ok_or(Error::<T>::InvalidUsdtValue)?
        .checked_div(&FixedU128::from(10_u128.pow(asset_from.exponent() as u32)))
        .ok_or(Error::<T>::BalanceOverflow)?;

      Ok(
        currency_wanted
          .saturating_mul(FixedU128::from(
            10_u128.pow(Asset::Tether.exponent() as u32),
          ))
          .into_inner()
          .checked_div(FixedU128::DIV)
          .ok_or(Error::<T>::InvalidUsdtValue)?,
      )
    }

    // Based on the value provided by Oracle, try to convert the asset balance to TDFY balance
    pub(crate) fn try_get_tide_value(
      currency_id: CurrencyId,
      amount: FixedU128,
    ) -> Result<Balance, DispatchError> {
      let asset_from: Asset = currency_id
        .try_into()
        .map_err(|_| Error::<T>::InvalidAsset)?;

      let order_book_price = if currency_id == CurrencyId::Tdfy {
        FixedU128::from(1)
      } else {
        Self::order_book_price(currency_id, CurrencyId::Tdfy)
      };

      if order_book_price.is_zero() {
        return Ok(0);
      }

      let currency_wanted = amount
        .checked_div(&order_book_price)
        .ok_or(Error::<T>::InvalidTdfyValue)?
        .checked_div(&FixedU128::from(10_u128.pow(asset_from.exponent() as u32)))
        .ok_or(Error::<T>::BalanceOverflow)?;

      Ok(
        currency_wanted
          .checked_mul(&FixedU128::from(10_u128.pow(Asset::Tdfy.exponent() as u32)))
          .ok_or(Error::<T>::InvalidTdfyValue)?
          .into_inner()
          .checked_div(FixedU128::DIV)
          .ok_or(Error::<T>::BalanceOverflow)?,
      )
    }

    // Based on the fee, try to select the highest matching sunrise pool
    pub(crate) fn try_select_first_eligible_sunrise_pool(
      fee: &Fee,
      currency_id: CurrencyId,
    ) -> Result<Option<SunriseSwapPool>, DispatchError> {
      // get all pools
      let current_usdt_trade_value = Self::try_get_usdt_value(currency_id, fee.amount.into())?;

      let mut all_pools = SunrisePools::<T>::get()
        .iter()
        // make sure there is enough transaction remaining in the pool
        .filter(|pool| pool.transactions_remaining > 0)
        // make sure there is enough TDFY remaining to fullfill this
        .filter(|pool| {
          pool.balance > 0
            && pool.balance
              >= Self::calculate_tide_reward_for_pool(pool.rebates, fee, currency_id)
                .unwrap_or_default()
        })
        // make sure the transaction amount value in USDT is higher
        .filter(|pool| pool.minimum_usdt_value >= current_usdt_trade_value)
        .map(|sunrise_pool| sunrise_pool.to_owned())
        .collect::<Vec<SunriseSwapPool>>();

      // sort descending by minimum usdt value
      all_pools.sort_by(|a, b| {
        b.minimum_usdt_value
          .partial_cmp(&a.minimum_usdt_value)
          .unwrap_or(sp_std::cmp::Ordering::Equal)
      });

      Ok(
        all_pools
          .first()
          .map(|sunrise_pool| sunrise_pool.to_owned()),
      )
    }

    // Initialize new era
    pub fn start_era() {
      ActiveEra::<T>::mutate(|active_era| {
        let new_index = active_era
          .as_ref()
          .map(|info| info.index.saturating_add(1))
          .unwrap_or(0);
        *active_era = Some(ActiveEraInfo::<T::BlockNumber> {
          index: new_index,
          // Set new active era start in next `on_finalize`. To guarantee usage of `Time`
          start_session_index: None,
          start_block: None,
          last_session_block: None,
          start: None,
        });
        new_index
      });
    }

    // Calculate the Sunrise rewards (TDFY balance) from the currency and the fee
    pub fn calculate_tide_reward_for_pool(
      rebates: FixedU128,
      fee: &Fee,
      currency_id: CurrencyId,
    ) -> Result<Balance, DispatchError> {
      let maximum_usdt_value = Asset::Tether.saturating_mul(10_000);
      let real_fee_with_rebates = (if fee.fee_usdt > maximum_usdt_value {
        FixedU128::from(Self::try_get_value_from_usdt(
          currency_id,
          maximum_usdt_value.into(),
        )?)
      } else {
        FixedU128::from(fee.fee)
      })
      .checked_mul(&rebates)
      .ok_or(Error::<T>::InvalidTdfyValue)?
      .into_inner()
      .checked_div(FixedU128::DIV)
      .ok_or(Error::<T>::BalanceOverflow)?;

      Self::try_get_tide_value(currency_id, real_fee_with_rebates.into())
    }
  }

  impl<T: Config> FeesExt<T::AccountId> for Pallet<T> {
    fn account_id() -> T::AccountId {
      T::FeesPalletId::get().into_account_truncating()
    }

    // FIXME: Would probably worth moving this into his own pallet?
    fn register_order_book_price(
      prices: Vec<(CurrencyId, CurrencyId, Balance)>,
    ) -> Result<(), DispatchError> {
      for (from, to, price) in prices {
        let asset_from: Asset = from.try_into().map_err(|_| Error::<T>::InvalidAsset)?;
        OrderBookPrice::<T>::insert(
          from,
          to,
          FixedU128::saturating_from_rational(price, asset_from.saturating_mul(1)),
        );
      }
      Ok(())
    }

    fn calculate_swap_fees(
      currency_id: CurrencyId,
      total_amount_before_fees: Balance,
      swap_type: SwapType,
      is_market_maker: bool,
    ) -> Fee {
      let fee = if is_market_maker {
        match swap_type {
          SwapType::Limit => T::MarketMakerLimitFeeAmount::get(),
          SwapType::Market => T::MarketMakerFeeAmount::get(),
        }
      } else {
        T::FeeAmount::get()
      } * total_amount_before_fees;

      Fee {
        amount: total_amount_before_fees,
        fee,
        fee_usdt: Self::try_get_usdt_value(currency_id, fee.into()).unwrap_or_default(),
      }
    }

    fn register_swap_fees(
      account_id: T::AccountId,
      currency_id: CurrencyId,
      total_amount_before_fees: Balance,
      swap_type: SwapType,
      is_market_maker: bool,
    ) -> Result<Fee, DispatchError> {
      let fee = match Self::active_era() {
        Some(current_era) => {
          let current_session = CurrentSession::<T>::get();
          let new_fee = Self::calculate_swap_fees(
            currency_id,
            total_amount_before_fees,
            swap_type,
            is_market_maker,
          );

          if let Some(sunrise_pool_available) =
            Self::try_select_first_eligible_sunrise_pool(&new_fee, currency_id)?
          {
            let real_fees_in_tide_with_rebates = Self::calculate_tide_reward_for_pool(
              sunrise_pool_available.rebates,
              &new_fee,
              currency_id,
            )?;
            // Update sunrise pool
            SunrisePools::<T>::try_mutate::<(), DispatchError, _>(|pools| {
              let sunrise_pool = pools
                .iter_mut()
                .find(|pool| pool.id == sunrise_pool_available.id)
                .ok_or(Error::<T>::InvalidSunrisePool)?;

              // Reduce pool balance
              sunrise_pool.balance = sunrise_pool
                .balance
                .saturating_sub(real_fees_in_tide_with_rebates);

              // Reduce number of transactions remaining for this pool
              sunrise_pool.transactions_remaining -= 1;

              Ok(())
            })?;

            // Increment reward for the account
            SunriseRewards::<T>::mutate(account_id.clone(), current_era.index, |rewards| {
              *rewards = rewards.saturating_add(real_fees_in_tide_with_rebates);
            });

            // Emit event
            Self::deposit_event(Event::<T>::SunriseRewarded {
              era_index: current_era.index,
              pool_id: sunrise_pool_available.id,
              account_id: account_id.clone(),
              reward: real_fees_in_tide_with_rebates,
            });
          }

          // Update fees pool for the current era / currency
          EraTotalFees::<T>::mutate_exists(
            current_era.index,
            currency_id,
            |current_currency_fee| {
              *current_currency_fee = Some(
                current_currency_fee
                  .as_ref()
                  .map(|current_fee| Fee {
                    amount: current_fee.amount.saturating_add(new_fee.amount),
                    fee: current_fee.fee.saturating_add(new_fee.fee),
                    fee_usdt: current_fee.fee_usdt.saturating_add(new_fee.fee_usdt),
                  })
                  .unwrap_or_else(|| new_fee.clone()),
              );
            },
          );

          // Update fees pool for the current session / currency
          SessionTotalFees::<T>::mutate_exists(
            current_session,
            currency_id,
            |current_currency_fee| {
              *current_currency_fee = Some(
                current_currency_fee
                  .as_ref()
                  .map(|current_fee| Fee {
                    amount: current_fee.amount.saturating_add(new_fee.amount),
                    fee: current_fee.fee.saturating_add(new_fee.fee),
                    fee_usdt: current_fee.fee_usdt.saturating_add(new_fee.fee_usdt),
                  })
                  .unwrap_or_else(|| new_fee.clone()),
              );
            },
          );

          // Update the total fees for the account
          AccountFees::<T>::try_mutate_exists::<u32, T::AccountId, (), DispatchError, _>(
            current_era.index,
            account_id,
            |account_fee_for_era| match account_fee_for_era {
              Some(account_fee) => {
                match account_fee
                  .iter_mut()
                  .find(|(found_currency_id, _)| *found_currency_id == currency_id)
                {
                  Some((_, current_fee)) => {
                    current_fee.amount = current_fee.amount.saturating_add(new_fee.amount);
                    current_fee.fee = current_fee.fee.saturating_add(new_fee.fee);
                    current_fee.fee_usdt = current_fee.fee_usdt.saturating_add(new_fee.fee_usdt);
                  }
                  None => {
                    account_fee
                      .try_push((currency_id, new_fee.clone()))
                      .map_err(|_| Error::<T>::AccountFeeOverflow)?;
                  }
                }
                Ok(())
              }
              None => {
                let bounded_vec: BoundedAccountFees = vec![(currency_id, new_fee.clone())]
                  .try_into()
                  .map_err(|_| Error::<T>::AccountFeeOverflow)?;
                *account_fee_for_era = Some(bounded_vec);
                Ok(())
              }
            },
          )?;

          new_fee
        }
        // No fees are taken as there is no active era
        None => Fee {
          amount: total_amount_before_fees,
          fee: 0,
          fee_usdt: 0,
        },
      };

      Ok(fee)
    }
  }
}
