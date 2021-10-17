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
  use frame_system::pallet_prelude::*;
  use sp_runtime::{traits::AccountIdConversion, Percent, Permill, SaturatedConversion};
  use tidefi_primitives::{
    pallet::FeesExt, ActiveEraInfo, Balance, BalanceInfo, BlockNumber, CurrencyId, EraIndex, Stake,
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
  pub type ActiveEra<T> = StorageValue<_, ActiveEraInfo>;

  /// The percentage on each trade to be taken as a network fee
  #[pallet::storage]
  #[pallet::getter(fn fees_percentage)]
  pub type FeePercentageAmount<T: Config> = StorageValue<_, Percent, ValueQuery>;

  /// The total fees for the era.
  /// If total hasn't been set or has been removed then 0 stake is returned.
  #[pallet::storage]
  #[pallet::getter(fn eras_total_fee)]
  pub type ErasTotalFee<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    EraIndex,
    Blake2_128Concat,
    CurrencyId,
    Balance,
    ValueQuery,
  >;

  /// Account fees for current era
  #[pallet::storage]
  #[pallet::getter(fn account_fees)]
  pub type AccountFees<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    CurrencyId,
    Blake2_128Concat,
    T::AccountId,
    Balance,
    ValueQuery,
  >;

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// The fees get redistributed successfully
    /// \[era last block, currency_id, amount\]
    FeesRedistribution(BlockNumber, CurrencyId, Balance),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {}

  // hooks
  #[pallet::hooks]
  impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
    fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
      // just return the weight of the on_finalize.
      T::DbWeight::get().reads(1)
    }

    fn on_finalize(current_block: BlockNumberFor<T>) {
      // Set the start of the first era.
      if let Some(mut active_era) = Self::active_era() {
        if active_era.start.is_none() {
          let now_as_millis_u64 = T::UnixTime::now().as_millis().saturated_into::<u64>();
          active_era.start = Some(now_as_millis_u64);
          // This write only ever happens once, we don't include it in the weight in
          // general
          ActiveEra::<T>::put(active_era);
        } 
        // calculate the end of era
      }
      // `on_finalize` weight is tracked in `on_initialize`
    }
  }

  impl<T: Config> Pallet<T> {
    fn start_era() {
      ActiveEra::<T>::mutate(|active_era| {
        let new_index = active_era.as_ref().map(|info| info.index + 1).unwrap_or(0);
        *active_era = Some(ActiveEraInfo {
          index: new_index,
          // Set new active era start in next `on_finalize`. To guarantee usage of `Time`
          start: None,
        });
        new_index
      });
    }

    fn end_era(active_era: ActiveEraInfo) {
      // Note: active_era_start can be None if end era is called during genesis config.
      if let Some(active_era_start) = active_era.start {
        let now_as_millis_u64 = T::UnixTime::now().as_millis().saturated_into::<u64>();

        let era_duration = (now_as_millis_u64 - active_era_start).saturated_into::<u64>();
        let all_fees_collected: Vec<(CurrencyId, Balance)> =
          ErasTotalFee::<T>::iter_prefix(active_era.index).collect();

        for (currency_id, balance) in all_fees_collected {
          // distribute to all accounts
          for (account_id, total_fee_paid) in AccountFees::<T>::iter_prefix(currency_id) {
            //T::CurrencyWrapr::transfer(currency_id, Self::account_id(), account_id, )
          }
        }
      }
    }
  }

  impl<T: Config> FeesExt<T::AccountId> for Pallet<T> {
    fn account_id() -> T::AccountId {
      T::FeesPalletId::get().into_account()
    }

    // we do not use the currency for now as all asset have same fees
    // but if we need to update in the future, we could simply use the currency id
    // and update the storage
    fn calculate_trading_fee(_currency_id: CurrencyId, amount: Balance) -> Balance {
      Self::fees_percentage() * amount
    }
  }
}
