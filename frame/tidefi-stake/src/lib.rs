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

pub(crate) const LOG_TARGET: &str = "tidefi::staking";

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
    traits::tokens::fungibles::{Inspect, Mutate, Transfer},
    BoundedVec, PalletId,
  };
  use frame_system::pallet_prelude::*;
  use sp_runtime::{traits::AccountIdConversion, ArithmeticError, Percent, Perquintill};
  use tidefi_primitives::{
    pallet::{AssetRegistryExt, SecurityExt, StakingExt},
    Balance, BalanceInfo, CurrencyId, Hash, SessionIndex, Stake,
  };

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config: frame_system::Config {
    /// Events
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    /// Pallet ID
    #[pallet::constant]
    type StakePalletId: Get<PalletId>;

    /// Unstake queue's capacity
    #[pallet::constant]
    type UnstakeQueueCap: Get<u32>;

    /// Maximum active stake / account
    #[pallet::constant]
    type StakeAccountCap: Get<u32>;

    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;

    /// Asset registry traits
    type AssetRegistry: AssetRegistryExt;

    /// Security traits
    type Security: SecurityExt<Self::AccountId, Self::BlockNumber>;

    /// Tidechain currency wrapper
    type CurrencyTidefi: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  /// Staking pool
  #[pallet::storage]
  #[pallet::getter(fn staking_pool)]
  pub type StakingPool<T: Config> = StorageMap<_, Blake2_128Concat, CurrencyId, Balance>;

  /// Staking rewards defined by the council
  #[pallet::storage]
  #[pallet::getter(fn staking_rewards)]
  pub type StakingPeriodRewards<T: Config> =
    StorageValue<_, Vec<(T::BlockNumber, Percent)>, ValueQuery>;

  /// The last session we should compound the account interests.
  #[pallet::storage]
  #[pallet::getter(fn interest_compound_last_session)]
  pub type InterestCompoundLastSession<T: Config> = StorageValue<_, SessionIndex, ValueQuery>;

  /// Manage which we should pay off to.
  #[pallet::storage]
  #[pallet::getter(fn unstake_queue)]
  pub type UnstakeQueue<T: Config> =
    StorageValue<_, BoundedVec<(T::AccountId, CurrencyId, Hash), T::UnstakeQueueCap>, ValueQuery>;

  /// Map from all pending stored sessions.
  // When all stake that are bounded for this sessions are compounded, they got removed from the map.
  // When the map is empty, the `do_next_compound_interest_operation` is not triggered.
  #[pallet::storage]
  #[pallet::getter(fn pending_sessions)]
  pub type PendingStoredSessions<T: Config> =
    CountedStorageMap<_, Blake2_128Concat, SessionIndex, ()>;

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
    Balance,
    ValueQuery,
  >;

  /// Account staking by CurrencyID
  #[pallet::storage]
  #[pallet::getter(fn account_stakes)]
  pub type AccountStakes<T: Config> = CountedStorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<Stake<Balance, T::BlockNumber>, T::StakeAccountCap>,
    ValueQuery,
  >;

  /// Genesis configuration
  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    pub staking_periods: Vec<(T::BlockNumber, Percent)>,
    pub runtime_marker: PhantomData<T>,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      // at 6 sec block length, we do ~ 14400 blocks / day
      Self {
        runtime_marker: PhantomData,
        staking_periods: vec![
          ((14400_u32 * 15_u32).into(), Percent::from_parts(2)),
          ((14400_u32 * 30_u32).into(), Percent::from_parts(3)),
          ((14400_u32 * 60_u32).into(), Percent::from_parts(4)),
          ((14400_u32 * 90_u32).into(), Percent::from_parts(5)),
        ],
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      StakingPeriodRewards::<T>::put(self.staking_periods.clone());
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// The assets get staked successfully
    Staked {
      request_id: Hash,
      account_id: T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
    },
    /// The assets get `unstaked` successfully
    Unstaked {
      request_id: Hash,
      account_id: T::AccountId,
      currency_id: CurrencyId,
      initial_amount: Balance,
      final_amount: Balance,
    },
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Duration doesn't exist on-chain.
    InvalidDuration,
    /// Exceeded unstake queue's capacity
    UnstakeQueueCapExceeded,
    /// Insufficient balance
    InsufficientBalance,
  }

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    /// Try to compute when chain is idle
    fn on_idle(_n: BlockNumberFor<T>, mut remaining_weight: Weight) -> Weight {
      let do_next_compound_interest_operation_weight =
        <T as frame_system::Config>::DbWeight::get().reads_writes(6, 6);

      // security if loop is get jammed somehow or prevent any overflow in tests
      let max_iter = 100;
      let mut current_iter = 0;

      loop {
        if remaining_weight > do_next_compound_interest_operation_weight
          && PendingStoredSessions::<T>::count() > 0
        {
          match Self::do_next_compound_interest_operation(remaining_weight) {
            Ok((real_weight_consumed, should_continue)) => {
              remaining_weight -= real_weight_consumed;

              if !should_continue {
                break;
              }
            }
            Err(err) => {
              log!(error, "Interest compounding failed {:?}", err);
              break;
            }
          };
        } else {
          break;
        }

        current_iter += 1;
        if current_iter >= max_iter {
          break;
        }
      }

      remaining_weight
    }
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Stake currency
    ///
    /// - `currency_id`: The currency to stake
    /// - `amount`: The amount to stake
    /// - `duration`: The duration is in numbers of blocks. (blocks are ~6seconds)
    ///
    /// Emits `Staked` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::stake())]
    pub fn stake(
      origin: OriginFor<T>,
      currency_id: CurrencyId,
      amount: Balance,
      duration: T::BlockNumber,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the transaction is signed
      let account_id = ensure_signed(origin)?;

      // 2. Make sure the duration exist on chain
      ensure!(
        StakingPeriodRewards::<T>::get()
          .into_iter()
          .find(|(iter_duration, _)| *iter_duration == duration)
          .is_some(),
        Error::<T>::InvalidDuration
      );

      // create unique hash
      let unique_stake_request_id = <T as Config>::Security::get_unique_id(account_id.clone());

      // 3. Transfer the funds into the staking pool
      T::CurrencyTidefi::can_withdraw(currency_id, &account_id, amount)
        .into_result()
        .map_err(|_| Error::<T>::InsufficientBalance)?;

      T::CurrencyTidefi::transfer(currency_id, &account_id, &Self::account_id(), amount, false)?;

      // 4. Update our `StakingPool` storage
      StakingPool::<T>::try_mutate(currency_id, |balance| -> DispatchResult {
        if let Some(b) = balance {
          *balance = Some(b.checked_add(amount).ok_or(ArithmeticError::Overflow)?);
        } else {
          *balance = Some(amount)
        }
        Ok(())
      })?;

      // 5. Insert the new staking
      let initial_block = <frame_system::Pallet<T>>::block_number();
      AccountStakes::<T>::mutate(account_id.clone(), |stake| -> DispatchResult {
        stake
          .try_push(Stake {
            currency_id,
            unique_id: unique_stake_request_id,
            last_session_index_compound: InterestCompoundLastSession::<T>::get(),
            initial_block,
            initial_balance: amount,
            principal: amount,
            duration,
          })
          .map_err(|_| DispatchError::Other("Invalid stake; eqd"))
      })?;

      // 6. Emit event on chain
      Self::deposit_event(Event::<T>::Staked {
        request_id: unique_stake_request_id,
        account_id,
        currency_id,
        amount,
      });

      Ok(().into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
      <T as pallet::Config>::StakePalletId::get().into_account()
    }

    // FIXME: require more tests to prevent any blocking on-chain
    #[inline]
    pub fn do_next_compound_interest_operation(
      max_weight: Weight,
    ) -> Result<(Weight, bool), DispatchError> {
      let weight_per_iteration = <T as frame_system::Config>::DbWeight::get().reads_writes(2, 2);
      let max_iterations = if weight_per_iteration == 0 {
        100
      } else {
        max_weight / weight_per_iteration
      };

      let pending_session_to_compound = PendingStoredSessions::<T>::count();
      if pending_session_to_compound == 0 {
        return Ok((weight_per_iteration, false));
      }

      let all_pending_sessions: Vec<SessionIndex> = PendingStoredSessions::<T>::iter()
        .map(|(key, _)| key)
        .collect();

      let mut current_iterations = 1;
      let last_session = InterestCompoundLastSession::<T>::get();

      log!(
        trace,
        "Running next compound interest operation for {} sessions.",
        all_pending_sessions.len()
      );

      // loop all stakes
      //for (account_id, currency_id, stake_details) in AccountStakes::<T>::iter() {
      for (account_id, account_stakes) in AccountStakes::<T>::iter() {
        //for (_, last_session_indexed, _) in account_stakes {
        for current_stake in account_stakes {
          let currency_id = current_stake.currency_id;
          let staking_pool_for_this_currency = StakingPool::<T>::get(currency_id).unwrap_or(0);
          if current_stake.last_session_index_compound <= last_session {
            let mut keep_going_in_loop = Some(current_stake.last_session_index_compound + 1);
            while let Some(session_to_index) = keep_going_in_loop {
              if all_pending_sessions.contains(&session_to_index) {
                let session_fee_for_currency =
                  SessionTotalFees::<T>::get(session_to_index, currency_id);
                AccountStakes::<T>::mutate(
                  account_id.clone(),
                  |staking_details| -> DispatchResult {
                    // loop trough all stake for this currency for this account
                    let mut final_stake = staking_details.clone();
                    for active_stake in final_stake.as_mut().iter_mut() {
                      // FIXME: we could probably find the closest reward
                      // but in theory this should never happens
                      let available_reward = StakingPeriodRewards::<T>::get()
                        .into_iter()
                        .find(|(duration, _)| *duration == active_stake.duration)
                        .map(|(_, reward)| reward)
                        .unwrap_or_else(Percent::zero)
                        * session_fee_for_currency;

                      // calculate proportional reward base on the stake pool
                      let staking_pool_percentage = Perquintill::from_rational(
                        active_stake.initial_balance,
                        staking_pool_for_this_currency,
                      );

                      let proportional_reward = staking_pool_percentage * available_reward;

                      active_stake.principal =
                        active_stake.principal.saturating_add(proportional_reward);

                      // update the last session index for this stake
                      active_stake.last_session_index_compound = session_to_index;

                      // increment our weighting
                      current_iterations += 1;

                      log!(
                        trace,
                        "Recomputed rewards for {:?} with new balance: {} (initial: {}) - i{}",
                        account_id,
                        active_stake.principal,
                        active_stake.initial_balance,
                        current_iterations,
                      )
                    }

                    *staking_details = final_stake;
                    Ok(())
                  },
                )?;
              }
              if session_to_index < last_session {
                keep_going_in_loop = Some(session_to_index + 1);
              } else {
                keep_going_in_loop = None;
              }

              if current_iterations >= max_iterations {
                keep_going_in_loop = None;
              }
            }
          }
        }
      }

      // FIXME: we should have a better draining
      PendingStoredSessions::<T>::remove_all();

      // FIXME: implement maximum iteration / should_continue = true
      Ok((current_iterations * weight_per_iteration, false))
    }

    #[inline]
    pub fn do_next_unstake_operation() -> Result<(), DispatchError> {
      // cost one more get but add a little security
      if Self::unstake_queue().is_empty() {
        return Ok(());
      }

      // get the front of the queue.
      // let (account_id, currency_id, unique_hash) = &Self::unstake_queue()[0];

      // get the staking details
      // AccountStakes::<T>::try_mutate(account_id, |all_stakes| -> DispatchResult { Ok(()) })?;

      // remove unstake request from queue
      Self::pop_unstake_task();

      Ok(())
    }

    #[inline]
    fn pop_unstake_task() {
      UnstakeQueue::<T>::mutate(|v| v.remove(0));
    }

    // Get all stakes for the account, serialized for quick RPC call
    pub fn get_account_stakes(
      account_id: &T::AccountId,
    ) -> Vec<(CurrencyId, Stake<BalanceInfo, T::BlockNumber>)> {
      let mut final_stakes = Vec::new();

      // we need to re-organize as our storage use a unique AccountId / CurrencyId key
      for account_stake in Self::account_stakes(account_id) {
        final_stakes.push((
          account_stake.currency_id,
          Stake {
            currency_id: account_stake.currency_id,
            last_session_index_compound: account_stake.last_session_index_compound,
            unique_id: account_stake.unique_id,
            initial_block: account_stake.initial_block,
            principal: BalanceInfo {
              amount: account_stake.principal,
            },
            initial_balance: BalanceInfo {
              amount: account_stake.initial_balance,
            },
            duration: account_stake.duration,
          },
        ));
      }

      final_stakes
    }
  }

  // implement the `StakingExt` functions
  impl<T: Config> StakingExt<T::AccountId> for Pallet<T> {
    fn account_id() -> T::AccountId {
      T::StakePalletId::get().into_account()
    }

    fn on_session_end(
      session_index: SessionIndex,
      session_trade_values: Vec<(CurrencyId, Balance)>,
    ) -> Result<(), DispatchError> {
      InterestCompoundLastSession::<T>::put(session_index);
      PendingStoredSessions::<T>::insert(session_index, ());
      for (currency_id, total_fees_for_the_session) in session_trade_values {
        SessionTotalFees::<T>::insert(session_index, currency_id, total_fees_for_the_session);
      }
      Ok(())
    }
  }
}
