#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

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
      tokens::fungibles::{Inspect, Mutate, Transfer},
      UnixTime,
    },
    PalletId,
  };

  use sp_runtime::{traits::AccountIdConversion, Percent, SaturatedConversion};
  use tidefi_primitives::{
    pallet::{FeesExt, SecurityExt},
    ActiveEraInfo, Balance, BlockNumber, CurrencyId, EraIndex, Fee,
  };

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config: frame_system::Config {
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

    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;

    /// Security traits
    type Security: SecurityExt<Self::AccountId, Self::BlockNumber>;

    /// Currency wrapr
    type CurrencyWrapr: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  /// The active era information, it holds index and start.
  ///
  /// The active era is the era being currently rewarded.
  #[pallet::storage]
  #[pallet::getter(fn active_era)]
  pub type ActiveEra<T: Config> = StorageValue<_, ActiveEraInfo<T::BlockNumber>>;

  /// The length of an era in block number.
  #[pallet::storage]
  #[pallet::getter(fn era_length)]
  pub type EraLength<T: Config> = StorageValue<_, T::BlockNumber>;

  /// The percentage on each trade to be taken as a network fee
  #[pallet::storage]
  #[pallet::getter(fn fee_percentage)]
  pub type FeePercentageAmount<T: Config> = StorageValue<_, Percent, ValueQuery>;

  /// The percentage of all fees for the each asset to re-distribute based on
  /// the trading volume for each account
  #[pallet::storage]
  #[pallet::getter(fn distribution_percentage)]
  pub type DistributionPercentage<T: Config> = StorageValue<_, Percent, ValueQuery>;

  /// The total fees for the era.
  /// If total hasn't been set or has been removed then 0 stake is returned.
  #[pallet::storage]
  #[pallet::getter(fn era_total_fees)]
  pub type EraTotalFees<T: Config> =
    StorageDoubleMap<_, Blake2_128Concat, EraIndex, Blake2_128Concat, CurrencyId, Fee, ValueQuery>;

  /// Account fees for current era
  #[pallet::storage]
  #[pallet::getter(fn account_fees)]
  pub type AccountFees<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    CurrencyId,
    Blake2_128Concat,
    T::AccountId,
    Fee,
    ValueQuery,
  >;

  /// Genesis configuration
  #[pallet::genesis_config]
  pub struct GenesisConfig {
    /// The percentage on each trade to be taken as a network fee. Default is 2%.
    pub fee_percentage: Percent,
    /// The percentage of all fees for the each asset to re-distribute based on
    /// the trading volume for each account. Default is 20%.
    pub distribution_percentage: Percent,
  }

  #[cfg(feature = "std")]
  impl Default for GenesisConfig {
    fn default() -> Self {
      Self {
        // Default network fee
        fee_percentage: Percent::from_parts(2),
        // Default distribution percentage
        distribution_percentage: Percent::from_parts(20),
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig {
    fn build(&self) {
      FeePercentageAmount::<T>::put(self.fee_percentage);
      DistributionPercentage::<T>::put(self.distribution_percentage);
    }
  }

  #[pallet::event]
  pub enum Event<T: Config> {
    /// The fees get redistributed successfully
    /// \[era last block, currency_id, amount\]
    Rewarded(BlockNumber, CurrencyId, Balance),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {}

  // hooks
  #[pallet::hooks]
  impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
    fn on_initialize(_now: T::BlockNumber) -> Weight {
      // just return the weight of the on_finalize.
      T::DbWeight::get().reads(2)
    }

    fn on_finalize(_current_block: T::BlockNumber) {
      if let Some(mut active_era) = Self::active_era() {
        let real_block = T::Security::get_current_block_count();
        match active_era.start_block {
          Some(start_block) => {
            let expected_end_block = start_block + Self::era_length().unwrap_or_default();
            // end of era
            if real_block >= expected_end_block {
              let _ = Self::end_era(active_era);
            }
          }
          None => {
            let now_as_millis_u64 = T::UnixTime::now().as_millis().saturated_into::<u64>();
            active_era.start = Some(now_as_millis_u64);
            active_era.start_block = Some(real_block);
            // This write only ever happens once, we don't include it in the weight in
            // general
            ActiveEra::<T>::put(active_era);
          }
        }
      }
      // `on_finalize` weight is tracked in `on_initialize`
    }
  }

  impl<T: Config> Pallet<T> {
    fn _start_era() {
      ActiveEra::<T>::mutate(|active_era| {
        let new_index = active_era.as_ref().map(|info| info.index + 1).unwrap_or(0);
        *active_era = Some(ActiveEraInfo::<T::BlockNumber> {
          index: new_index,
          // Set new active era start in next `on_finalize`. To guarantee usage of `Time`
          start_block: None,
          start: None,
        });
        new_index
      });
    }

    fn end_era(active_era: ActiveEraInfo<T::BlockNumber>) -> Result<(), DispatchError> {
      // Note: active_era_start can be None if end era is called during genesis config.
      if let Some(active_era_start) = active_era.start {
        let now_as_millis_u64 = T::UnixTime::now().as_millis().saturated_into::<u64>();

        let _era_duration = (now_as_millis_u64 - active_era_start).saturated_into::<u64>();
        let all_fees_collected: Vec<(CurrencyId, Fee)> =
          EraTotalFees::<T>::iter_prefix(active_era.index).collect();

        for (currency_id, fees_details_collected_in_era) in all_fees_collected {
          let total_amount_for_currency = fees_details_collected_in_era.amount;
          let total_fees_collected_for_currency = fees_details_collected_in_era.fee;

          // The amount of tokens in each monthly distribution will
          // be equal to `DistributionPercentage` of `CurrencyId` trading revenue (fees collected).
          let revenue_for_current_currency =
            Self::distribution_percentage() * total_fees_collected_for_currency;

          // distribute to all accounts
          for (account_id, account_fee_for_currency) in AccountFees::<T>::iter_prefix(currency_id) {
            let total_transfer_in_era_for_account = account_fee_for_currency.amount;
            let total_token_for_current_account = (total_transfer_in_era_for_account
              / total_amount_for_currency)
              * revenue_for_current_currency;

            // FIXME: Convert this amount in TIDE and transfer them from
            // this account maybe?
            let _total_tide_token = total_token_for_current_account;

            T::CurrencyWrapr::transfer(
              CurrencyId::Tide,
              &Self::account_id(),
              &account_id,
              total_token_for_current_account,
              true,
            )?;
          }
        }
      }

      Ok(())
    }
  }

  impl<T: Config> FeesExt<T::AccountId> for Pallet<T> {
    fn account_id() -> T::AccountId {
      T::FeesPalletId::get().into_account()
    }

    // we do not use the currency for now as all asset have same fees
    // but if we need to update in the future, we could simply use the currency id
    // and update the storage
    fn calculate_trading_fees(_currency_id: CurrencyId, total_amount_before_fees: Balance) -> Fee {
      Fee {
        amount: total_amount_before_fees,
        fee: Self::fee_percentage() * total_amount_before_fees,
      }
    }

    fn register_trading_fees(
      account_id: T::AccountId,
      currency_id: CurrencyId,
      total_amount_before_fees: Balance,
    ) -> Fee {
      match Self::active_era() {
        Some(current_era) => {
          let new_fee = Fee {
            amount: total_amount_before_fees,
            fee: Self::fee_percentage() * total_amount_before_fees,
          };

          // Update fees pool for the current era / currency
          EraTotalFees::<T>::mutate_exists(
            current_era.index,
            currency_id,
            |current_currency_fee| {
              *current_currency_fee = Some(
                current_currency_fee
                  .as_ref()
                  .map(|current_fee| Fee {
                    amount: current_fee.amount + new_fee.amount,
                    fee: current_fee.fee + new_fee.fee,
                  })
                  .unwrap_or_else(|| new_fee.clone()),
              );
            },
          );

          // Update the total fees for the account
          AccountFees::<T>::mutate_exists(currency_id, account_id, |account_fee_for_currency| {
            *account_fee_for_currency = Some(
              account_fee_for_currency
                .as_ref()
                .map(|current_fee| Fee {
                  amount: current_fee.amount + new_fee.amount,
                  fee: current_fee.fee + new_fee.fee,
                })
                .unwrap_or_else(|| new_fee.clone()),
            );
          });

          new_fee
        }
        None => Fee {
          amount: total_amount_before_fees,
          fee: 0,
        },
      }
    }
  }
}
