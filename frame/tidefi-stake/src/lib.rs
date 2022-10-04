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

pub(crate) const LOG_TARGET: &str = "tidefi::staking";

mod migrations;

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
      StorageVersion,
    },
    BoundedVec, PalletId,
  };
  use frame_system::pallet_prelude::*;
  use sp_runtime::{
    traits::{AccountIdConversion, Saturating},
    ArithmeticError, Percent, Perquintill,
  };
  use tidefi_primitives::{
    pallet::{AssetRegistryExt, SecurityExt, StakingExt},
    Balance, BalanceInfo, CurrencyId, Hash, SessionIndex, Stake, StakeCurrencyMeta, StakeStatus,
  };

  /// The current storage version.
  const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

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

    /// Maximum number of periods defined
    #[pallet::constant]
    type StakingRewardCap: Get<u32>;

    /// Number of block to wait before unstake if forced.
    #[pallet::constant]
    type BlocksForceUnstake: Get<Self::BlockNumber>;

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
  #[pallet::storage_version(STORAGE_VERSION)]
  pub struct Pallet<T>(_);

  /// Staking pool
  #[pallet::storage]
  #[pallet::getter(fn staking_pool)]
  pub type StakingPool<T: Config> = StorageMap<_, Blake2_128Concat, CurrencyId, Balance>;

  /// Staking rewards defined by the council
  #[pallet::storage]
  #[pallet::getter(fn staking_rewards)]
  pub type StakingPeriodRewards<T: Config> =
    StorageValue<_, BoundedVec<(T::BlockNumber, Percent), T::StakingRewardCap>, ValueQuery>;

  /// Staking metadata defined by the council (minimum and maximum stake amount)
  #[pallet::storage]
  #[pallet::getter(fn staking_meta)]
  pub type StakingCurrencyMeta<T: Config> =
    StorageMap<_, Blake2_128Concat, CurrencyId, StakeCurrencyMeta<Balance>>;

  /// The percentage of fee when unstake is done before the ending.
  #[pallet::storage]
  #[pallet::getter(fn unstake_fee)]
  pub type UnstakeFee<T: Config> = StorageValue<_, Percent, ValueQuery>;

  /// The last session we should compound the account interests.
  #[pallet::storage]
  #[pallet::getter(fn interest_compound_last_session)]
  pub type InterestCompoundLastSession<T: Config> = StorageValue<_, SessionIndex, ValueQuery>;

  /// Manage which we should pay off to.
  #[pallet::storage]
  #[pallet::getter(fn unstake_queue)]
  pub type UnstakeQueue<T: Config> = StorageValue<
    _,
    BoundedVec<(T::AccountId, Hash, T::BlockNumber), T::UnstakeQueueCap>,
    ValueQuery,
  >;

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
    pub staking_meta: Vec<(CurrencyId, StakeCurrencyMeta<Balance>)>,
    pub unstake_fee: Percent,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      // at 6 sec block length, we do ~ 14400 blocks / day
      Self {
        // 1%
        unstake_fee: Percent::from_parts(1),
        staking_periods: vec![
          // FIXME: Remove the 15 minutes after our tests
          (150_u32.into(), Percent::from_parts(1)),
          ((14400_u32 * 15_u32).into(), Percent::from_parts(2)),
          ((14400_u32 * 30_u32).into(), Percent::from_parts(3)),
          ((14400_u32 * 60_u32).into(), Percent::from_parts(4)),
          ((14400_u32 * 90_u32).into(), Percent::from_parts(5)),
        ],
        staking_meta: Vec::new(),
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      let bounded_periods: BoundedVec<(T::BlockNumber, Percent), T::StakingRewardCap> =
        self.staking_periods.clone().try_into().expect("too much periods");

      StakingPeriodRewards::<T>::put(bounded_periods);
      UnstakeFee::<T>::put(self.unstake_fee.clone());

      for (currency_id, staking_meta) in self.staking_meta.clone() {
        StakingCurrencyMeta::<T>::insert(currency_id, staking_meta);
      }
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
      duration: T::BlockNumber,
    },
    /// The assets unstaking has been queued
    UnstakeQueued {
      request_id: Hash,
      account_id: T::AccountId,
    },
    /// The assets get `unstaked` successfully
    Unstaked {
      request_id: Hash,
      account_id: T::AccountId,
      currency_id: CurrencyId,
      initial_balance: Balance,
      final_balance: Balance,
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
    /// Invalid stake request ID
    InvalidStakeId,
    /// Unstake is not ready
    UnstakingNotReady,
    /// Something went wrong with fees transfer
    TransferFeesFailed,
    /// Something went wrong with funds transfer
    TransferFailed,
    /// Staking pool is empty
    NotEnoughInPoolToUnstake,
    /// The staked amount is below the minimum stake amount for this currency.
    AmountTooSmall,
    /// The staked amount is above the maximum stake amount for this currency.
    AmountTooLarge,
  }

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
      migrations::migrate_to_v1::<T, Self>()
    }

    /// Try to compute when chain is idle
    fn on_idle(_n: BlockNumberFor<T>, mut remaining_weight: Weight) -> Weight {
      let do_next_compound_interest_operation_weight =
        <T as frame_system::Config>::DbWeight::get().reads_writes(6, 6);
      let do_next_unstake_operation_weight =
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
        } else if remaining_weight > do_next_unstake_operation_weight
          && !Self::unstake_queue().is_empty()
        {
          match Self::do_next_unstake_operation(remaining_weight) {
            Ok((real_weight_consumed, should_continue)) => {
              remaining_weight -= real_weight_consumed;

              if !should_continue {
                break;
              }
            }
            Err(err) => {
              log!(error, "Unstake queue failed {:?}", err);
              break;
            }
          };
        } else {
          break;
        }

        current_iter += 1;
        if current_iter >= max_iter {
          log!(
            warn,
            "Max iter reached; something is wrong with `on_idle` logic; overflow prevented"
          );
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

      // 3. If we have the metadata of this currency, make sure the amount isn't too low or too high
      if let Some(currency_meta) = StakingCurrencyMeta::<T>::get(currency_id) {
        ensure!(
          amount >= currency_meta.minimum_amount,
          Error::<T>::AmountTooSmall
        );
        ensure!(
          amount <= currency_meta.maximum_amount,
          Error::<T>::AmountTooLarge
        );
      }

      // create unique hash
      let unique_stake_request_id =
        Self::add_account_stake(&account_id, currency_id, amount, duration)?;

      // 4. Emit event on chain
      Self::deposit_event(Event::<T>::Staked {
        request_id: unique_stake_request_id,
        account_id,
        currency_id,
        amount,
        duration,
      });

      Ok(().into())
    }

    /// Unstake
    ///
    /// - `stake_id`: Unique Stake ID
    /// - `force_unstake`: Unstake with extra fees, even if the staking is not expired
    ///
    /// Emits `Unstaked` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::unstake())]
    pub fn unstake(
      origin: OriginFor<T>,
      stake_id: Hash,
      force_unstake: bool,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the transaction is signed
      let account_id = ensure_signed(origin)?;

      // 2. Get Staking request for this user
      let stake =
        Self::get_account_stake(&account_id, stake_id).ok_or(Error::<T>::InvalidStakeId)?;

      // 3. Check the expiration and if we are forcing it (queue)
      let expected_block_expiration = stake.initial_block.saturating_add(stake.duration);
      let staking_is_expired = T::Security::get_current_block_count() >= expected_block_expiration;

      if staking_is_expired {
        // we can process to unstaking immediately
        Self::process_unstake(&account_id, stake_id)?;
        Self::deposit_event(Event::<T>::Unstaked {
          request_id: stake_id,
          account_id,
          currency_id: stake.currency_id,
          initial_balance: stake.initial_balance,
          final_balance: stake.principal,
        });
      } else {
        ensure!(force_unstake, Error::<T>::UnstakingNotReady);

        // FIXME: Validate not already queued

        // we should add to unstaking queue and take immeditately the extra fees
        // for the queue storage

        // take the fee
        // FIXME: would be great to convert to TDFY
        let unstaking_fee = Self::unstake_fee() * stake.initial_balance;
        T::CurrencyTidefi::can_withdraw(stake.currency_id, &account_id, unstaking_fee)
          .into_result()
          .map_err(|_| Error::<T>::InsufficientBalance)?;

        let expected_block_end =
          T::Security::get_current_block_count().saturating_add(T::BlocksForceUnstake::get());
        UnstakeQueue::<T>::try_append((account_id.clone(), stake_id, expected_block_end))
          .map_err(|_| Error::<T>::UnstakeQueueCapExceeded)?;

        // update `AccountStakes` status
        AccountStakes::<T>::try_mutate(account_id.clone(), |stakes| -> DispatchResult {
          let stake = stakes
            .iter_mut()
            .find(|stake| stake.unique_id == stake_id)
            .ok_or(Error::<T>::InvalidStakeId)?;
          stake.status = StakeStatus::PendingUnlock(expected_block_end);
          Ok(())
        })?;

        T::CurrencyTidefi::transfer(
          stake.currency_id,
          &account_id,
          &Self::account_id(),
          unstaking_fee,
          true,
        )
        .map_err(|_| Error::<T>::TransferFeesFailed)?;

        Self::deposit_event(Event::<T>::UnstakeQueued {
          request_id: stake_id,
          account_id,
        });
      }

      Ok(().into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
      <T as pallet::Config>::StakePalletId::get().into_account_truncating()
    }

    pub fn add_account_stake(
      account_id: &T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
      duration: T::BlockNumber,
    ) -> Result<Hash, DispatchError> {
      // create unique hash
      let unique_stake_request_id = <T as Config>::Security::get_unique_id(account_id.clone());

      // 3. Transfer the funds into the staking pool
      T::CurrencyTidefi::can_withdraw(currency_id, account_id, amount)
        .into_result()
        .map_err(|_| Error::<T>::InsufficientBalance)?;

      T::CurrencyTidefi::transfer(currency_id, account_id, &Self::account_id(), amount, false)?;

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
      let initial_block = T::Security::get_current_block_count();
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
            status: StakeStatus::Staked,
          })
          .map_err(|_| DispatchError::Other("Invalid stake; eqd"))
      })?;

      Ok(unique_stake_request_id)
    }

    fn get_account_stake(
      account_id: &T::AccountId,
      stake_id: Hash,
    ) -> Option<Stake<Balance, T::BlockNumber>> {
      AccountStakes::<T>::get(account_id)
        .into_iter()
        .find(|stake| stake.unique_id == stake_id)
    }

    fn process_unstake(account_id: &T::AccountId, stake_id: Hash) -> DispatchResult {
      let current_stake =
        Self::get_account_stake(&account_id, stake_id).ok_or(Error::<T>::InvalidStakeId)?;
      AccountStakes::<T>::try_mutate_exists(account_id, |account_stakes| match account_stakes {
        None => Err(Error::<T>::InvalidStakeId),
        Some(stakes) => {
          T::CurrencyTidefi::can_withdraw(
            current_stake.currency_id,
            &Self::account_id(),
            current_stake.principal,
          )
          .into_result()
          .map_err(|_| Error::<T>::InsufficientBalance)?;

          StakingPool::<T>::try_mutate(current_stake.currency_id, |balance| -> DispatchResult {
            if let Some(b) = balance {
              *balance = Some(
                b.checked_sub(current_stake.principal)
                  .ok_or(ArithmeticError::Underflow)?,
              )
            }
            Ok(())
          })
          .map_err(|_| Error::<T>::NotEnoughInPoolToUnstake)?;

          T::CurrencyTidefi::transfer(
            current_stake.currency_id,
            &Self::account_id(),
            &account_id,
            current_stake.principal,
            false,
          )
          .map_err(|_| Error::<T>::TransferFailed)?;

          if stakes.len() > 1 {
            stakes.retain(|stake| stake.unique_id != stake_id);
          } else {
            *account_stakes = None;
          }

          Ok(())
        }
      })?;

      Ok(())
    }

    // FIXME: move to offchain worker
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
            let mut keep_going_in_loop =
              Some(current_stake.last_session_index_compound.saturating_add(1));
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
                      if T::Security::get_current_block_count()
                        <= current_stake
                          .initial_block
                          .saturating_add(current_stake.duration)
                      {
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
                      }
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
                keep_going_in_loop = Some(session_to_index.saturating_add(1));
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
      let _ = PendingStoredSessions::<T>::clear(u32::MAX, None);

      // FIXME: implement maximum iteration / should_continue = true
      Ok((current_iterations * weight_per_iteration, false))
    }

    #[inline]
    pub fn do_next_unstake_operation(max_weight: Weight) -> Result<(Weight, bool), DispatchError> {
      let weight_per_iteration = <T as frame_system::Config>::DbWeight::get().reads_writes(2, 2);
      let unstake_queue = Self::unstake_queue();
      if unstake_queue.is_empty() {
        return Ok((weight_per_iteration, false));
      }

      log!(
        trace,
        "Running next queued unstake operation with a queue size of {}.",
        unstake_queue.len()
      );

      let max_iterations = if weight_per_iteration == 0 {
        1
      } else {
        max_weight / weight_per_iteration
      };

      let mut keep_going_in_loop = Some(0);
      let mut total_weight = weight_per_iteration;
      while let Some(current_iterations) = keep_going_in_loop {
        Self::do_unstake_queue_front()?;

        total_weight += weight_per_iteration;
        if current_iterations >= max_iterations || current_iterations >= unstake_queue.len() as u64
        {
          keep_going_in_loop = None;
        } else {
          keep_going_in_loop = Some(current_iterations.saturating_add(1));
        }
      }

      Ok((total_weight, false))
    }

    fn do_unstake_queue_front() -> Result<(), DispatchError> {
      let unstake_queue = Self::unstake_queue();
      if unstake_queue.is_empty() {
        return Ok(());
      }

      let (account_id, stake_id, expiration) = &unstake_queue[0];

      if T::Security::get_current_block_count() < *expiration {
        return Ok(());
      }

      Self::process_unstake(&account_id, *stake_id)?;
      UnstakeQueue::<T>::mutate(|v| v.remove(0));

      Ok(())
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
            status: account_stake.status,
          },
        ));
      }

      final_stakes
    }
  }

  // implement the `StakingExt` functions
  impl<T: Config> StakingExt<T::AccountId> for Pallet<T> {
    fn account_id() -> T::AccountId {
      T::StakePalletId::get().into_account_truncating()
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
