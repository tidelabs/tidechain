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

pub mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod migrations;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

pub(crate) const LOG_TARGET: &str = "tidefi::fees";

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
      StorageVersion, UnixTime,
    },
    PalletId,
  };
  use sp_runtime::{
    traits::{AccountIdConversion, Saturating},
    Percent, Permill, SaturatedConversion,
  };
  use sp_std::vec;
  use tidefi_primitives::{
    pallet::{FeesExt, SecurityExt, StakingExt, SunriseExt},
    ActiveEraInfo, Balance, CurrencyId, EraIndex, Fee, SessionIndex, SwapType,
  };

  /// The current storage version.
  const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

  type BoundedAccountFees = BoundedVec<(CurrencyId, Fee), ConstU32<1_000>>;

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config: frame_system::Config {
    /// Events
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

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

    /// Retail user swap fee
    #[pallet::constant]
    type FeeAmount: Get<Permill>;

    /// Market maker market order fee
    #[pallet::constant]
    type MarketMakerFeeAmount: Get<Permill>;

    /// Market maker limit order fee
    #[pallet::constant]
    type MarketMakerLimitFeeAmount: Get<Permill>;

    /// Security traits
    type Security: SecurityExt<Self::AccountId, Self::BlockNumber>;

    /// Tidefi stake traits
    type Staking: StakingExt<Self::AccountId>;

    /// Tidefi sunrise traits
    type Sunrise: SunriseExt<Self::AccountId, Self::BlockNumber>;

    /// Tidechain currency wrapper
    type CurrencyTidefi: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

    /// The origin which may forcibly update the fee and distribution percentage
    type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  #[pallet::storage_version(STORAGE_VERSION)]
  pub struct Pallet<T>(_);

  /// The active era information, it holds index and start.
  ///
  /// The active era is the era being currently rewarded.
  #[pallet::storage]
  pub type ActiveEra<T: Config> = StorageValue<_, ActiveEraInfo<T::BlockNumber>>;

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

  /// Genesis configuration
  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    pub phantom: PhantomData<T>,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        phantom: PhantomData,
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
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
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Account fees overflow
    AccountFeeOverflow,
    /// Balance overflow
    BalanceOverflow,
    /// Invalid TDFY value in the order book
    InvalidTdfyValue,
  }

  // hooks
  #[pallet::hooks]
  impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
    fn on_initialize(_now: T::BlockNumber) -> Weight {
      if Self::should_finalize_current_session() {
        <T as Config>::WeightInfo::on_finalize(
          // predict the size of the new queue to be created in `pallet_tidefi_staking`
          T::Staking::account_stakes_size() as u32,
          // predict the size of the `session_fees_by_currency`
          SessionTotalFees::<T>::iter_prefix(CurrentSession::<T>::get()).count() as u32,
        )
        .saturating_add(T::DbWeight::get().reads(6))
      } else {
        // just return the weight of the `should_finalize_current_session`
        T::DbWeight::get().reads(4)
      }
    }

    fn on_finalize(_current_block: T::BlockNumber) {
      if let Some(mut active_era) = Self::current_era() {
        let real_block = T::Security::get_current_block_count();
        match active_era.start_block {
          Some(start_block) => {
            // determine when the session
            let session_start_block = active_era.last_session_block.unwrap_or(start_block);
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
              if let Err(err) = T::Staking::on_session_end(
                current_session,
                session_fees_by_currency.clone(),
                Self::account_id(),
              ) {
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

  impl<T: Config> Pallet<T> {
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

    pub(crate) fn should_finalize_current_session() -> bool {
      if let Some(active_era) = Self::current_era() {
        let real_block = T::Security::get_current_block_count();
        if let Some(start_block) = active_era.start_block {
          let session_start_block = active_era.last_session_block.unwrap_or(start_block);
          let expected_end_block_for_session =
            session_start_block.saturating_add(T::BlocksPerSession::get());

          return real_block == expected_end_block_for_session;
        }
      }

      false
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
  }

  impl<T: Config> FeesExt<T::AccountId, T::BlockNumber> for Pallet<T> {
    fn era_blocks_count() -> T::BlockNumber {
      T::BlocksPerSession::get()
        .saturating_mul(T::BlockNumber::from(T::SessionsPerEra::get() as u32))
    }

    fn account_id() -> T::AccountId {
      T::FeesPalletId::get().into_account_truncating()
    }

    fn current_era() -> Option<ActiveEraInfo<T::BlockNumber>> {
      ActiveEra::<T>::get()
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
        fee_tdfy: T::Sunrise::try_get_tdfy_value(currency_id, fee).unwrap_or_default(),
      }
    }

    fn register_swap_fees(
      account_id: T::AccountId,
      currency_id: CurrencyId,
      total_amount_before_fees: Balance,
      swap_type: SwapType,
      is_market_maker: bool,
    ) -> Result<Fee, DispatchError> {
      let fee = match Self::current_era() {
        Some(current_era) => {
          let current_session = CurrentSession::<T>::get();
          let new_fee = Self::calculate_swap_fees(
            currency_id,
            total_amount_before_fees,
            swap_type,
            is_market_maker,
          );

          T::Sunrise::try_allocate_rewards_for_swap(
            &account_id,
            current_era.index,
            &new_fee,
            currency_id,
          )?;

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
                    fee_tdfy: current_fee.fee_tdfy.saturating_add(new_fee.fee_tdfy),
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
                    fee_tdfy: current_fee.fee_tdfy.saturating_add(new_fee.fee_tdfy),
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
                    current_fee.fee_tdfy = current_fee.fee_tdfy.saturating_add(new_fee.fee_tdfy);
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
          fee_tdfy: 0,
        },
      };

      Ok(fee)
    }
  }
}
