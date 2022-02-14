#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

pub(crate) const LOG_TARGET: &str = "tidefi::fees";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: crate::LOG_TARGET,
			concat!("[{:?}] 💸 ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
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
      UnixTime,
    },
    PalletId,
  };
  use frame_system::pallet_prelude::*;

  use sp_runtime::{
    traits::{AccountIdConversion, Saturating},
    Percent, SaturatedConversion,
  };
  use tidefi_primitives::{
    pallet::{FeesExt, SecurityExt, StakingExt},
    ActiveEraInfo, Balance, CurrencyId, EraIndex, Fee, SessionIndex,
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

    /// Number of sessions per era
    #[pallet::constant]
    type SessionsPerEra: Get<SessionIndex>;

    /// Number of sessions to keep in archive
    #[pallet::constant]
    type SessionsArchive: Get<SessionIndex>;

    /// Number of block per session
    #[pallet::constant]
    type BlocksPerSession: Get<Self::BlockNumber>;

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
  pub struct Pallet<T>(_);

  /// The active era information, it holds index and start.
  ///
  /// The active era is the era being currently rewarded.
  #[pallet::storage]
  #[pallet::getter(fn active_era)]
  pub type ActiveEra<T: Config> = StorageValue<_, ActiveEraInfo<T::BlockNumber>>;

  /// The current session of the era.
  #[pallet::storage]
  #[pallet::getter(fn current_session)]
  pub type CurrentSession<T: Config> = StorageValue<_, SessionIndex, ValueQuery>;

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

  /// Map from all stored sessions.
  #[pallet::storage]
  #[pallet::getter(fn stored_sessions)]
  pub type StoredSessions<T: Config> = StorageMap<_, Blake2_128Concat, SessionIndex, ()>;

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
  pub struct GenesisConfig<T: Config> {
    /// The percentage on each trade to be taken as a network fee. Default is 2%.
    pub fee_percentage: Percent,
    /// The percentage of all fees for the each asset to re-distribute based on
    /// the trading volume for each account. Default is 20%.
    pub distribution_percentage: Percent,
    pub runtime_marker: PhantomData<T>,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        // Default network fee
        fee_percentage: Percent::from_parts(2),
        // Default distribution percentage
        distribution_percentage: Percent::from_parts(20),
        runtime_marker: PhantomData,
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      CurrentSession::<T>::put(1);
      FeePercentageAmount::<T>::put(self.fee_percentage);
      DistributionPercentage::<T>::put(self.distribution_percentage);
      ActiveEra::<T>::put(ActiveEraInfo::<T::BlockNumber> {
        index: 1,
        // Set new active era start in next `on_finalize`. To guarantee usage of `Time`
        start_block: None,
        start_session_index: None,
        last_session_block: None,
        start: None,
      });
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    DistributionPercentageUpdated(Percent),
    FeesPercentageUpdated(Percent),
    SessionEnded {
      era_index: EraIndex,
      session_index: SessionIndex,
      session_fees_by_currency: Vec<(CurrencyId, Balance)>,
    },
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
            // determine when the last session ended
            let session_start_block = match active_era.last_session_block {
              Some(last_session_block) => last_session_block,
              None => start_block,
            };

            let expected_end_block_for_session =
              session_start_block.saturating_add(T::BlocksPerSession::get());

            // end of session
            if real_block >= expected_end_block_for_session {
              let current_session = CurrentSession::<T>::get();
              log!(
                info,
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
              ActiveEra::<T>::put(active_era);

              // let expected_end_session_index_for_era = start_session_index.saturating_add(T::SessionsPerEra::get());

              // if current_session >= expected_end_session_index_for_era {
              // FIXME: Finalize tokenomics
              // if let Err(err) = T::Staking::_end_era(current_session) {
              //   log!(error, "Can't close the era {:?}", err);
              // }
              // }
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
    #[pallet::weight(0)]
    pub fn set_distribution_percentage(
      origin: OriginFor<T>,
      percentage: Percent,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure this is signed by `account_id`
      T::ForceOrigin::ensure_origin(origin)?;

      // 2. Update oracle status
      DistributionPercentage::<T>::set(percentage);

      // 3. Emit event on chain
      Self::deposit_event(Event::<T>::DistributionPercentageUpdated(percentage));

      Ok(().into())
    }

    #[pallet::weight(0)]
    pub fn set_fees_percentage(
      origin: OriginFor<T>,
      percentage: Percent,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure this is signed by `account_id`
      T::ForceOrigin::ensure_origin(origin)?;

      // 2. Update oracle status
      FeePercentageAmount::<T>::set(percentage);

      // 3. Emit event on chain
      Self::deposit_event(Event::<T>::FeesPercentageUpdated(percentage));

      Ok(().into())
    }
  }

  impl<T: Config> Pallet<T> {
    pub(crate) fn drain_old_sessions() {
      let current_session = CurrentSession::<T>::get();
      for (session, _) in StoredSessions::<T>::iter() {
        if session < current_session.saturating_sub(T::SessionsArchive::get()) {
          // delete the session
          SessionTotalFees::<T>::remove_prefix(session, None);
          StoredSessions::<T>::remove(session);
        }
      }
    }

    pub fn start_era() {
      ActiveEra::<T>::mutate(|active_era| {
        let new_index = active_era.as_ref().map(|info| info.index + 1).unwrap_or(0);
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

    pub(crate) fn _end_era(active_era: ActiveEraInfo<T::BlockNumber>) -> Result<(), DispatchError> {
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

            T::CurrencyTidefi::transfer(
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
    fn calculate_swap_fees(_currency_id: CurrencyId, total_amount_before_fees: Balance) -> Fee {
      Fee {
        amount: total_amount_before_fees,
        fee: Self::fee_percentage() * total_amount_before_fees,
      }
    }

    fn register_swap_fees(
      account_id: T::AccountId,
      currency_id: CurrencyId,
      total_amount_before_fees: Balance,
    ) -> Fee {
      match Self::active_era() {
        Some(current_era) => {
          let current_session = CurrentSession::<T>::get();
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

          // Update fees pool for the current session / currency
          SessionTotalFees::<T>::mutate_exists(
            current_session,
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
