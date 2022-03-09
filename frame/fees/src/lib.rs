#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

mod migrations;

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
  use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaChaRng,
  };
  use sp_runtime::{
    traits::{AccountIdConversion, Saturating},
    Percent, Permill, SaturatedConversion,
  };
  use sp_std::{borrow::ToOwned, vec};
  use tidefi_primitives::{
    assets::{Asset, USDT},
    pallet::{FeesExt, SecurityExt, StakingExt},
    ActiveEraInfo, Balance, CurrencyId, EraIndex, Fee, SessionIndex, SunriseSwapPool,
  };

  /// The current storage version.
  const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

  type BoundedAccountFees = BoundedVec<(CurrencyId, Fee), ConstU32<1_000>>;
  type BoundedSunrisePools = BoundedVec<SunriseSwapPool, ConstU32<6>>;

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

    /// Sunrise pool account id
    #[pallet::constant]
    type SunriseAccountId: Get<Self::AccountId>;

    /// Number of sessions to keep in archive
    #[pallet::constant]
    type SessionsArchive: Get<SessionIndex>;

    /// Number of block per session
    #[pallet::constant]
    type BlocksPerSession: Get<Self::BlockNumber>;

    /// Number of sessions to keep in archive
    #[pallet::constant]
    type FeeAmount: Get<Permill>;

    /// Number of sessions to keep in archive
    #[pallet::constant]
    type MarketMakerFeeAmount: Get<Permill>;

    /// Number of sessions to keep in archive
    #[pallet::constant]
    type DistributionPercentage: Get<Permill>;

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

  /// Tide price of the orderbook reported by oracle every X minutes at the current market price.
  /// We keep in sync order book of USDT values for our sunrise pool.
  ///
  /// CurrencyId → USDT
  /// USDT → TIDE
  ///
  /// To get current TIDE USDT value;
  /// ```rust
  /// Self::order_book_price(CurrencyId::Tide, CurrencyId::Wrapped(4))
  /// ```
  ///
  #[pallet::storage]
  #[pallet::getter(fn order_book_price)]
  pub type OrderBookPrice<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    CurrencyId,
    Blake2_128Concat,
    CurrencyId,
    Balance,
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
    pub runtime_marker: PhantomData<T>,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        runtime_marker: PhantomData,
        sunrise_swap_pools: vec![
          SunriseSwapPool {
            id: 1,
            minimum_usdt_value: 0,
            transactions_remaining: 1_000,
            balance: Asset::Tide.saturating_mul(67_200_000),
            rebates: Permill::from_rational(100_u32, 100_u32),
          },
          SunriseSwapPool {
            id: 2,
            // 100 USDT minimum value
            minimum_usdt_value: Asset::Tether.saturating_mul(100),
            transactions_remaining: 1_000,
            balance: Asset::Tide.saturating_mul(57_600_000),
            // 125%
            rebates: Permill::from_rational(125_u32, 100_u32),
          },
          SunriseSwapPool {
            id: 3,
            // 1_000 USDT minimum value
            minimum_usdt_value: Asset::Tether.saturating_mul(1_000),
            transactions_remaining: 1_000,
            balance: Asset::Tide.saturating_mul(57_600_000),
            // 150%
            rebates: Permill::from_rational(150_u32, 100_u32),
          },
          SunriseSwapPool {
            id: 4,
            // 10_000 USDT minimum value
            minimum_usdt_value: Asset::Tether.saturating_mul(10_000),
            transactions_remaining: 100,
            balance: Asset::Tide.saturating_mul(38_400_000),
            // 150%
            rebates: Permill::from_rational(150_u32, 100_u32),
          },
          SunriseSwapPool {
            id: 5,
            // 50_000 USDT minimum value
            minimum_usdt_value: Asset::Tether.saturating_mul(50_000),
            transactions_remaining: 100,
            balance: Asset::Tide.saturating_mul(19_200_000),
            // 200%
            rebates: Permill::from_rational(200_u32, 100_u32),
          },
          SunriseSwapPool {
            id: 6,
            // 100_000 USDT minimum value
            minimum_usdt_value: Asset::Tether.saturating_mul(100_000),
            transactions_remaining: 100,
            balance: Asset::Tide.saturating_mul(9_600_000),
            // 300%
            rebates: Permill::from_rational(300_u32, 100_u32),
          },
        ],
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
    SunriseReward {
      era_index: EraIndex,
      pool_id: u8,
      account_id: T::AccountId,
      reward: Balance,
    },
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Invalid sunrise pool
    InvalidSunrisePool,
    /// Account fees overflow
    AccountFeeOverflow,
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

    pub(crate) fn sunrise_pool_for_fee(
      fee: Fee,
      currency_id: CurrencyId,
    ) -> Option<SunriseSwapPool> {
      // get all pools
      let current_tide_price = Self::order_book_price(CurrencyId::Wrapped(4), CurrencyId::Tide);
      let current_usdt_trade_value =
        fee.amount * Self::order_book_price(currency_id, CurrencyId::Wrapped(4));

      let mut all_pools = SunrisePools::<T>::get()
        .iter()
        // make sure there is enough transaction remaining in the pool
        .filter(|pool| pool.transactions_remaining > 0)
        // make sure there is enough tide remaining to fullfill this
        .filter(|pool| pool.balance > 0 && pool.balance > (fee.fee_usdt * current_tide_price))
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

      all_pools
        .first()
        .map(|sunrise_pool| sunrise_pool.to_owned())
    }

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

    fn build_era_account_fees(era: EraIndex, max_weight: Weight) -> Weight {
      // let process maximum 1000 account at a time
      let all_account_fees: Vec<(T::AccountId, BoundedAccountFees)> =
        AccountFees::<T>::iter_prefix(era).take(1_000).collect();

      let mut weight_used = <T as frame_system::Config>::DbWeight::get().reads(1);

      if all_account_fees.len() == 0 {
        return weight_used;
      }

      // The amount of remaining weight under which we stop processing messages
      let threshold_weight = 100_000;

      // we create a shuffle of index, to prevent queue blocking
      let mut shuffled = Self::create_shuffle(all_account_fees.len());
      let mut shuffle_index = 0;
      let mut weight_available = 0;

      while shuffle_index < shuffled.len()
        && max_weight.saturating_sub(weight_used) >= threshold_weight
      {
        let index = shuffled[shuffle_index];
        let account_id = &all_account_fees[index].0;
        let account_fees = &all_account_fees[index].1;

        if weight_available != max_weight {
          // The speed to which the available weight approaches the maximum weight. A lower number
          // results in a faster progression. A value of 1 makes the entire weight available initially.
          let weight_restrict_decay = 2;
          // Get incrementally closer to freeing up max_weight for first round.
          // For the second round we unlock all weight. If we come close enough
          // on the first round to unlocking everything, then we do so.
          if shuffle_index < all_account_fees.len() {
            weight_available += (max_weight - weight_available) / (weight_restrict_decay + 1);
            if weight_available + threshold_weight > max_weight {
              weight_available = max_weight;
            }
          } else {
            weight_available = max_weight;
          }
        }

        // loop all currency for this account
        for (currency_id, fee) in account_fees.clone() {
          // `T::DistributionPercentage`% of EraTotalFees paid
          let currency_total_fee_pool =
            T::DistributionPercentage::get() * EraTotalFees::<T>::get(era, currency_id).fee;
        }
      }

      weight_used
    }

    // Create a shuffled vector the size of `len` with random keys
    fn create_shuffle(len: usize) -> Vec<usize> {
      // Create a shuffled order for use to iterate through.
      // Not a great random seed, but good enough for our purposes.
      let seed = frame_system::Pallet::<T>::parent_hash();
      let seed = <[u8; 32]>::decode(&mut sp_runtime::traits::TrailingZeroInput::new(
        seed.as_ref(),
      ))
      .expect("input is padded with zeroes; qed");
      let mut rng = ChaChaRng::from_seed(seed);
      let mut shuffled = (0..len).collect::<Vec<_>>();
      for i in 0..len {
        let j = (rng.next_u32() as usize) % len;
        let a = shuffled[i];
        shuffled[i] = shuffled[j];
        shuffled[j] = a;
      }
      shuffled
    }

    pub(crate) fn _end_era(active_era: ActiveEraInfo<T::BlockNumber>) -> Result<(), DispatchError> {
      Ok(())
    }
  }

  impl<T: Config> FeesExt<T::AccountId> for Pallet<T> {
    fn account_id() -> T::AccountId {
      T::FeesPalletId::get().into_account()
    }

    fn calculate_swap_fees(
      currency_id: CurrencyId,
      total_amount_before_fees: Balance,
      is_market_maker: bool,
    ) -> Fee {
      let fee = if is_market_maker {
        T::MarketMakerFeeAmount::get()
      } else {
        T::FeeAmount::get()
      } * total_amount_before_fees;

      // get the fee value in USDT
      let fee_usdt = fee * Self::order_book_price(currency_id, CurrencyId::Wrapped(USDT));

      Fee {
        amount: total_amount_before_fees,
        fee,
        fee_usdt,
      }
    }

    fn register_swap_fees(
      account_id: T::AccountId,
      currency_id: CurrencyId,
      total_amount_before_fees: Balance,
      is_market_maker: bool,
    ) -> Result<Fee, DispatchError> {
      let fee = match Self::active_era() {
        Some(current_era) => {
          let current_session = CurrentSession::<T>::get();
          let new_fee =
            Self::calculate_swap_fees(currency_id, total_amount_before_fees, is_market_maker);

          if let Some(sunrise_pool_available) =
            Self::sunrise_pool_for_fee(new_fee.clone(), currency_id)
          {
            let fee_in_usdt_with_rebates = sunrise_pool_available.rebates * new_fee.fee_usdt;
            let real_fees_in_tide_with_rebates =
            // maximum 10k USDT in fees calculated
            if fee_in_usdt_with_rebates > Asset::Tether.saturating_mul(10_000) {
                Asset::Tether.saturating_mul(10_000)
              } else {
                fee_in_usdt_with_rebates
              } * Self::order_book_price(currency_id, CurrencyId::Tide);

            // Update sunrise pool
            SunrisePools::<T>::try_mutate::<(), DispatchError, _>(|pools| {
              let sunrise_pool = pools
                .iter_mut()
                .find(|pool| pool.id == sunrise_pool_available.id)
                .ok_or(Error::<T>::InvalidSunrisePool)?;

              sunrise_pool.balance = sunrise_pool
                .balance
                .saturating_sub(real_fees_in_tide_with_rebates);
              sunrise_pool.transactions_remaining -= 1;

              Ok(())
            })?;

            // Increment reward for the account
            SunriseRewards::<T>::mutate(account_id.clone(), current_era.index, |rewards| {
              *rewards = rewards.saturating_add(real_fees_in_tide_with_rebates);
            });

            // Emit event
            Self::deposit_event(Event::<T>::SunriseReward {
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
