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

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

pub(crate) const LOG_TARGET: &str = "tidefi::sunrise";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: $crate::LOG_TARGET,
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
      StorageVersion,
    },
    PalletId,
  };
  use sp_arithmetic::traits::Zero;
  use sp_runtime::{
    traits::{AccountIdConversion, CheckedMul, Saturating},
    FixedPointNumber, FixedU128,
  };
  use sp_std::borrow::ToOwned;
  use tidefi_primitives::{
    assets::Asset,
    pallet::{SecurityExt, SunriseExt},
    AssetId, Balance, CurrencyId, EraIndex, Fee, OnboardingRebates, SunriseSwapPool,
  };

  /// The current storage version.
  const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

  type BoundedPools = BoundedVec<SunriseSwapPool, ConstU32<6>>;

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config: frame_system::Config {
    /// Events
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    /// Pallet ID
    #[pallet::constant]
    type SunrisePalletId: Get<PalletId>;

    /// Number of blocks to wait before allowing users to claim their sunrise rewards, after an era is completed.
    #[pallet::constant]
    type Cooldown: Get<Self::BlockNumber>;

    /// Maximum rewards for sunrise pool before the applied rewards.
    /// Example, 10_000_000_000_000_000 with a 125% rewards,
    /// the maximum applied will be `12_500_000_000_000_000` TDFY
    #[pallet::constant]
    type MaximumRewardPerSwap: Get<Balance>;

    /// For each tier, leftover funds will be allocated to to this tier
    #[pallet::constant]
    type LeftoverSwapRebates: Get<FixedU128>;

    /// Security traits
    type Security: SecurityExt<Self::AccountId, Self::BlockNumber>;

    /// Tidechain currency wrapper
    type CurrencyTidefi: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  #[pallet::storage_version(STORAGE_VERSION)]
  pub struct Pallet<T>(_);

  /// The active sunrise tier availables.
  #[pallet::storage]
  #[pallet::getter(fn sunrise_pools)]
  pub type Pools<T: Config> = StorageValue<_, BoundedPools, ValueQuery>;

  /// The balance available as left-over from the pools.
  #[pallet::storage]
  #[pallet::getter(fn pools_left_over)]
  pub type PoolsLeftOverBalance<T: Config> = StorageValue<_, Balance, ValueQuery>;

  /// The active onboarding rebates (gas refunds on-deposit)
  #[pallet::storage]
  #[pallet::getter(fn onboarding)]
  pub type Onboarding<T: Config> = StorageValue<_, OnboardingRebates, ValueQuery>;

  /// TDFY price of each wrapped asset, reported by Oracle every X blocks.
  ///
  /// Exchange rate for 1 `AssetId` vs 1 TDFY
  #[pallet::storage]
  #[pallet::getter(fn wrapped_asset_value)]
  pub type ExchangeRateInTdfy<T: Config> =
    StorageMap<_, Blake2_128Concat, AssetId, FixedU128, ValueQuery>;

  /// Account fees for current era
  #[pallet::storage]
  #[pallet::getter(fn sunrise_rewards)]
  pub type Rewards<T: Config> = StorageDoubleMap<
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
    pub swap_pools: Vec<SunriseSwapPool>,
    pub onboarding_rebates: Option<OnboardingRebates>,
    pub phantom: PhantomData<T>,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        phantom: PhantomData,
        swap_pools: Vec::new(),
        onboarding_rebates: None,
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      let bounded_sunrise_pool: BoundedPools = self.swap_pools.clone().try_into().unwrap();
      Pools::<T>::put(bounded_sunrise_pool);

      if let Some(onboarding_rebates) = &self.onboarding_rebates {
        Onboarding::<T>::put(onboarding_rebates);
      }

      // Create Fee account
      let account_id = <Pallet<T>>::account_id();
      let min = T::CurrencyTidefi::minimum_balance(CurrencyId::Tdfy);
      if T::CurrencyTidefi::reducible_balance(CurrencyId::Tdfy, &account_id, false) < min {
        if let Err(err) = T::CurrencyTidefi::mint_into(CurrencyId::Tdfy, &account_id, min) {
          log!(
            error,
            "Unable to mint sunrise pallet minimum balance: {:?}",
            err
          );
        }
      }
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    SunriseRewarded {
      era_index: EraIndex,
      pool_id: Option<u8>,
      account_id: T::AccountId,
      reward: Balance,
    },
    SunriseClaimed {
      era_index: EraIndex,
      account_id: T::AccountId,
      reward: Balance,
    },
    OnboardingRebatesApplied {
      account_id: T::AccountId,
      currency_id: CurrencyId,
      initial_amount: Balance,
      rebate: Balance,
    },
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Invalid sunrise pool
    InvalidSunrisePool,
    /// Invalid asset
    InvalidAsset,
    /// Balance overflow
    BalanceOverflow,
    /// Invalid TDFY value in the order book
    InvalidTdfyValue,
    /// There is no rewards available for this account on this era
    NoRewardsAvailable,
    /// There is no rebates available to process the gas refund
    NoRebatesAvailable,
  }

  impl<T: Config> Pallet<T> {
    // Based on the fee, try to select the highest matching sunrise pool
    pub(crate) fn try_select_first_eligible_sunrise_pool(
      fee: &Fee,
      currency_id: CurrencyId,
    ) -> Result<Option<SunriseSwapPool>, DispatchError> {
      // get all pools
      let current_tdfy_trade_value = Self::try_get_tdfy_value(currency_id, fee.amount)?;

      let mut all_pools = Pools::<T>::get()
        .iter()
        // make sure there is enough transaction remaining in the pool
        .filter(|pool| pool.transactions_remaining > 0)
        // make sure there is enough tdfy's remaining to fullfill this
        .filter(|pool| {
          pool.balance > 0
            && pool.balance
              >= Self::calculate_rebates_on_fees_paid(pool.rebates, fee).unwrap_or_default()
        })
        .filter(|pool| pool.minimum_tdfy_value <= current_tdfy_trade_value)
        .map(|sunrise_pool| sunrise_pool.to_owned())
        .collect::<Vec<SunriseSwapPool>>();

      // sort descending by minimum TDFy's value
      all_pools.sort_by(|a, b| {
        b.minimum_tdfy_value
          .partial_cmp(&a.minimum_tdfy_value)
          .unwrap_or(sp_std::cmp::Ordering::Equal)
      });

      Ok(
        all_pools
          .first()
          .map(|sunrise_pool| sunrise_pool.to_owned()),
      )
    }

    // Calculate the rebates based on the fees paid
    pub fn calculate_rebates_on_fees_paid(
      rebates: FixedU128,
      fee: &Fee,
    ) -> Result<Balance, DispatchError> {
      let maximum_tdfy_value = T::MaximumRewardPerSwap::get();
      let real_fee_with_rebates_in_tdfy = FixedU128::from(if fee.fee_tdfy > maximum_tdfy_value {
        maximum_tdfy_value
      } else {
        fee.fee_tdfy
      })
      .checked_mul(&rebates)
      .ok_or(Error::<T>::InvalidTdfyValue)?
      .into_inner()
      .checked_div(FixedU128::DIV)
      .ok_or(Error::<T>::BalanceOverflow)?;

      Ok(real_fee_with_rebates_in_tdfy)
    }

    // convert a FixedU128 to TDFY
    pub fn convert_fixed_balance_to_tdfy_balance(
      fixed_balance: FixedU128,
    ) -> Result<Balance, DispatchError> {
      fixed_balance
        .checked_mul(&10_u128.pow(Asset::Tdfy.exponent() as u32).into())
        .ok_or(Error::<T>::InvalidTdfyValue)?
        .into_inner()
        .checked_div(FixedU128::DIV)
        .ok_or(Error::<T>::BalanceOverflow)
        .map_err(Into::into)
    }

    pub fn get_next_onboarding_rebates(
      amount_in_tdfy: Balance,
      onboarding_rebate: &OnboardingRebates,
    ) -> Result<Balance, DispatchError> {
      ensure!(
        onboarding_rebate.available_amount >= amount_in_tdfy,
        Error::<T>::NoRebatesAvailable
      );

      let amount_already_assigned = onboarding_rebate
        .initial_amount
        .saturating_sub(onboarding_rebate.available_amount);

      // The first 18 million TDFY of onboarding rebates will be paid out to fully rebate at 100%.
      if amount_already_assigned <= Asset::Tdfy.saturating_mul(18_000_000) {
        return Ok(amount_in_tdfy);
      }

      // The remaining rebates will follow a degressive schedule where each incoming transaction is rebated
      // to the same percentage amount as the remaining funds in the onboarding pool.

      // This degressive schedule ensures that those who use and support the project early will be exponentially
      // rewarded while there is still enough for everyone over a long period of time.

      let reward_ratio = FixedU128::saturating_from_rational(
        onboarding_rebate.available_amount,
        onboarding_rebate.initial_amount,
      );

      let amount =
        FixedU128::saturating_from_rational(amount_in_tdfy, Asset::Tdfy.saturating_mul(1))
          .saturating_mul(reward_ratio);

      Self::convert_fixed_balance_to_tdfy_balance(amount)
    }
  }

  impl<T: Config> SunriseExt<T::AccountId, T::BlockNumber> for Pallet<T> {
    fn account_id() -> T::AccountId {
      T::SunrisePalletId::get().into_account_truncating()
    }

    fn try_get_tdfy_value(currency_id: CurrencyId, amount: u128) -> Result<u128, DispatchError> {
      let asset_from: Asset = currency_id
        .try_into()
        .map_err(|_| Error::<T>::InvalidAsset)?;

      let order_book_price = match currency_id {
        CurrencyId::Tdfy => FixedU128::from(1),
        CurrencyId::Wrapped(asset_id) => Self::wrapped_asset_value(asset_id),
      };

      if order_book_price.is_zero() {
        return Ok(0);
      }

      let amount = FixedU128::saturating_from_rational(amount, asset_from.saturating_mul(1))
        .saturating_mul(order_book_price);

      Self::convert_fixed_balance_to_tdfy_balance(amount)
    }

    fn register_exchange_rate(prices: Vec<(AssetId, Balance)>) -> Result<(), DispatchError> {
      for (from, price_in_tdfy_for_one_unit_of_asset) in prices {
        ExchangeRateInTdfy::<T>::insert(
          from,
          FixedU128::saturating_from_rational(
            price_in_tdfy_for_one_unit_of_asset,
            Asset::Tdfy.saturating_mul(1),
          ),
        );
      }
      Ok(())
    }

    fn try_allocate_rewards_for_swap(
      account_id: &T::AccountId,
      era_index: EraIndex,
      fee: &Fee,
      currency_id: CurrencyId,
    ) -> Result<Option<Balance>, DispatchError> {
      if let Some(sunrise_pool_available) =
        Self::try_select_first_eligible_sunrise_pool(fee, currency_id)?
      {
        let real_fees_in_tdfy_with_rebates =
          Self::calculate_rebates_on_fees_paid(sunrise_pool_available.rebates, &fee)?;
        // Update sunrise pool
        Pools::<T>::try_mutate::<(), DispatchError, _>(|pools| {
          let sunrise_pool = pools
            .iter_mut()
            .find(|pool| pool.id == sunrise_pool_available.id)
            .ok_or(Error::<T>::InvalidSunrisePool)?;

          // Reduce pool balance
          sunrise_pool.balance = sunrise_pool
            .balance
            .saturating_sub(real_fees_in_tdfy_with_rebates);

          // Reduce number of transactions remaining for this pool
          sunrise_pool.transactions_remaining -= 1;

          // we've reached the end of the pool, move the balance to the left-over pool
          if sunrise_pool.transactions_remaining == 0 && sunrise_pool.balance > 0 {
            PoolsLeftOverBalance::<T>::mutate(|left_over| {
              *left_over = left_over.saturating_add(sunrise_pool.balance);
            });
            sunrise_pool.balance = 0;
          }

          Ok(())
        })?;

        // Increment reward for the account
        Rewards::<T>::mutate(account_id.clone(), era_index, |rewards| {
          *rewards = rewards.saturating_add(real_fees_in_tdfy_with_rebates);
        });

        // Emit event
        Self::deposit_event(Event::<T>::SunriseRewarded {
          era_index,
          pool_id: Some(sunrise_pool_available.id),
          account_id: account_id.clone(),
          reward: real_fees_in_tdfy_with_rebates,
        });

        Ok(Some(real_fees_in_tdfy_with_rebates))
      } else {
        // check if we have some leftover that can be used
        let available_left_over = Self::pools_left_over();
        let real_fees_in_tdfy_with_rebates =
          Self::calculate_rebates_on_fees_paid(T::LeftoverSwapRebates::get(), &fee)?;

        if available_left_over >= real_fees_in_tdfy_with_rebates {
          // Increment reward for the account
          Rewards::<T>::mutate(account_id.clone(), era_index, |rewards| {
            *rewards = rewards.saturating_add(real_fees_in_tdfy_with_rebates);
          });

          // Reduce leftover
          PoolsLeftOverBalance::<T>::mutate(|left_over| {
            *left_over = left_over.saturating_sub(real_fees_in_tdfy_with_rebates);
          });

          // Emit event
          Self::deposit_event(Event::<T>::SunriseRewarded {
            era_index,
            pool_id: None,
            account_id: account_id.clone(),
            reward: real_fees_in_tdfy_with_rebates,
          });

          Ok(Some(real_fees_in_tdfy_with_rebates))
        } else {
          Ok(None)
        }
      }
    }

    fn try_refund_gas_for_deposit(
      account_id: &T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
    ) -> Result<Option<Balance>, DispatchError> {
      let amount_in_tdfy = Self::try_get_tdfy_value(currency_id, amount)?;
      Onboarding::<T>::try_mutate(|onboarging_rebates| {
        // get the onboarding rebates
        let rebate = Self::get_next_onboarding_rebates(amount_in_tdfy, onboarging_rebates)?;
        if rebate.is_zero() {
          return Ok(None);
        }

        // transfer funds
        T::CurrencyTidefi::transfer(
          CurrencyId::Tdfy,
          &Self::account_id(),
          account_id,
          rebate,
          true,
        )?;

        Self::deposit_event(Event::<T>::OnboardingRebatesApplied {
          account_id: account_id.clone(),
          currency_id,
          initial_amount: amount,
          rebate,
        });

        onboarging_rebates.available_amount =
          onboarging_rebates.available_amount.saturating_sub(rebate);

        Ok(Some(rebate))
      })
    }

    fn try_claim_sunrise_rewards(
      account_id: &T::AccountId,
      era_index: EraIndex,
    ) -> Result<(), DispatchError> {
      Rewards::<T>::try_mutate_exists(account_id, era_index, |found_reward| {
        match found_reward {
          Some(reward) => {
            if reward.is_zero() {
              return Err(Error::<T>::NoRewardsAvailable.into());
            }

            // transfer funds
            T::CurrencyTidefi::transfer(
              CurrencyId::Tdfy,
              &Self::account_id(),
              account_id,
              *reward,
              true,
            )?;

            // emit event
            Self::deposit_event(Event::<T>::SunriseClaimed {
              era_index,
              account_id: account_id.clone(),
              reward: *reward,
            });
            // delete storage
            *found_reward = None;

            log!(
              debug,
              "Sunrise reward claimed for {:?} successfully.",
              account_id
            );
            Ok(())
          }
          None => Err(Error::<T>::NoRewardsAvailable.into()),
        }
      })
    }

    fn cooldown_blocks_count() -> T::BlockNumber {
      T::Cooldown::get()
    }
  }
}
