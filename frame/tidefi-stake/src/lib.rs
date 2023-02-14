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

pub mod migrations;

pub mod weights;
pub use weights::*;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

use frame_system::ensure_root;

pub(crate) const LOG_TARGET: &str = "tidefi::staking";

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
    pallet_prelude::{DispatchResultWithPostInfo, *},
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
  use sp_std::vec;
  use tidefi_primitives::{
    pallet::{AssetRegistryExt, SecurityExt, StakingExt},
    Balance, BalanceInfo, CurrencyId, Hash, SessionIndex, Stake, StakeCurrencyMeta, StakeStatus,
  };

  /// The current storage version.
  const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

  #[derive(Eq, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen, Clone, Debug)]
  pub enum BatchType {
    Unstake,
    Compound,
  }

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config: frame_system::Config {
    /// Events
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    /// Pallet ID
    #[pallet::constant]
    type StakePalletId: Get<PalletId>;

    /// Maximum active stake / account
    #[pallet::constant]
    type StakeAccountCap: Get<u32>;

    /// Maximum number of periods defined
    #[pallet::constant]
    type StakingRewardCap: Get<u32>;

    /// Number of block to wait before unstake if forced.
    #[pallet::constant]
    type BlocksForceUnstake: Get<Self::BlockNumber>;

    /// Batch size.
    ///
    /// This many accounts and unstake are processed in each on_idle` request.
    type BatchSize: Get<u32>;

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

  /// Map from all pending stored sessions.
  // When all stake that are bounded for this sessions are compounded, they got removed from the map.
  // When the map is empty, the `do_next_compound_interest_operation` is not triggered.
  // SessionIndex (amount distributed)
  #[pallet::storage]
  #[pallet::getter(fn pending_sessions)]
  pub type PendingStoredSessions<T: Config> = CountedStorageMap<
    _,
    Blake2_128Concat,
    SessionIndex,
    BoundedVec<(CurrencyId, Balance), T::StakingRewardCap>,
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

  #[pallet::storage]
  pub type QueueCompound<T: Config> =
    CountedStorageMap<_, Blake2_128Concat, T::AccountId, SessionIndex>;

  #[pallet::storage]
  pub type QueueUnstake<T: Config> =
    CountedStorageMap<_, Blake2_128Concat, Hash, (T::AccountId, T::BlockNumber)>;

  /// Operator account
  #[pallet::storage]
  #[pallet::getter(fn operator_account_id)]
  pub type OperatorAccountId<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

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
      let bounded_periods: BoundedVec<(T::BlockNumber, Percent), T::StakingRewardCap> = self
        .staking_periods
        .clone()
        .try_into()
        .expect("too much periods");

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
    /// Batch finished, number of session or unstaking processed.
    BatchFinished { size: u32, kind: BatchType },
    /// Batch compound, number of accounts
    BatchCompound { size: u32 },
    /// Session finished
    SessionFinished {
      session_index: SessionIndex,
      pool: Vec<(CurrencyId, Balance)>,
      operator: Vec<(CurrencyId, Balance)>,
    },
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Duration doesn't exist on-chain.
    InvalidDuration,
    /// Insufficient balance
    InsufficientBalance,
    /// Invalid stake request ID
    InvalidStakeId,
    /// Invalid session
    InvalidSession,
    /// Unstake is not ready
    UnstakingNotReady,
    /// Something went wrong with fees transfer
    TransferFeesFailed,
    /// Something went wrong with funds transfer
    TransferFailed,
    /// Staking pool is empty
    NotEnoughInPoolToUnstake,
    /// Stake is reduced
    StakeIsReduced,
    /// The staked amount is below the minimum stake amount for this currency.
    AmountTooSmall,
    /// The staked amount is above the maximum stake amount for this currency.
    AmountTooLarge,
  }

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    /// Try to compute when chain is idle
    fn on_idle(_n: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
      let unstake_queue_size = QueueUnstake::<T>::count();
      let compound_queue_size = QueueCompound::<T>::count();

      let do_next_compound_interest_operation_weight =
        <T as Config>::WeightInfo::on_idle_compound(compound_queue_size.into()).saturating_add(
          T::DbWeight::get().reads_writes(compound_queue_size.into(), compound_queue_size.into()),
        );

      let do_next_unstake_operation_weight =
        <T as Config>::WeightInfo::on_idle_unstake(unstake_queue_size.into()).saturating_add(
          T::DbWeight::get().reads_writes(unstake_queue_size.into(), unstake_queue_size.into()),
        );

      if remaining_weight > do_next_compound_interest_operation_weight
        && PendingStoredSessions::<T>::count() > 0
      {
        Self::do_on_idle_compound(remaining_weight);
      } else if remaining_weight > do_next_unstake_operation_weight && unstake_queue_size > 0 {
        Self::do_on_idle_unstake(remaining_weight);
      }

      remaining_weight
    }
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Set Staking Operator Account
    /// TODO: Benchmark weight
    #[pallet::weight(0)]
    pub fn set_operator_account_id(
      origin: OriginFor<T>,
      new_operator_account_id: T::AccountId,
    ) -> DispatchResultWithPostInfo {
      ensure_root(origin)?;
      OperatorAccountId::<T>::put(new_operator_account_id);

      Ok(().into())
    }

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
        Self::do_process_unstake(&account_id, stake_id)?;
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
        QueueUnstake::<T>::insert(stake_id, (account_id.clone(), expected_block_end));

        // update `AccountStakes` status
        AccountStakes::<T>::try_mutate(account_id.clone(), |stakes| -> DispatchResult {
          let stake = stakes
            .iter_mut()
            .find(|stake| stake.unique_id == stake_id)
            .ok_or(Error::<T>::InvalidStakeId)?;
          stake.status = StakeStatus::PendingUnlock(expected_block_end);
          Ok(())
        })?;

        // Pay unstaking fees to operator account instead of staking pool
        T::CurrencyTidefi::transfer(
          stake.currency_id,
          &account_id,
          &Self::operator_account(),
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

    pub fn operator_account() -> T::AccountId {
      let operator_account = match Self::operator_account_id() {
        Some(account_id) => account_id,
        None => Self::account_id(),
      };
      operator_account
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

    fn do_process_unstake(account_id: &T::AccountId, stake_id: Hash) -> DispatchResult {
      let current_stake =
        Self::get_account_stake(&account_id, stake_id).ok_or(Error::<T>::InvalidStakeId)?;
      AccountStakes::<T>::try_mutate_exists(account_id, |account_stakes| match account_stakes {
        None => Err(Error::<T>::InvalidStakeId),
        Some(stakes) => {
          T::CurrencyTidefi::can_withdraw(
            current_stake.currency_id,
            &Self::account_id(),
            current_stake.initial_balance,
          )
          .into_result()
          .map_err(|_| Error::<T>::InsufficientBalance)?;

          StakingPool::<T>::try_mutate(current_stake.currency_id, |balance| -> DispatchResult {
            if let Some(b) = balance {
              *balance = Some(
                b.checked_sub(current_stake.initial_balance)
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

    fn do_account_compound(
      account_id: &T::AccountId,
      session_fee_by_currency: &Vec<(SessionIndex, Vec<(CurrencyId, Balance)>)>,
    ) -> DispatchResult {
      AccountStakes::<T>::try_mutate(account_id, |staking_details| -> DispatchResult {
        // distribute reward for each session for each stake
        staking_details
          .iter_mut()
          .filter(|stake| {
            T::Security::get_current_block_count()
              <= stake.initial_block.saturating_add(stake.duration)
          })
          .for_each(|stake| {
            session_fee_by_currency.iter().for_each(
              |(session_to_index, collected_fees_by_currency)| {
                // do not compound if it's already processed
                if stake.last_session_index_compound >= *session_to_index {
                  return;
                }

                let session_fee_for_currency = collected_fees_by_currency
                  .into_iter()
                  .find_map(|(currency_id, fees_collected)| {
                    if *currency_id == stake.currency_id {
                      Some(*fees_collected)
                    } else {
                      None
                    }
                  })
                  .unwrap_or_default();

                // FIXME: we could probably find the closest reward
                // but in theory this should never happens
                let available_reward = StakingPeriodRewards::<T>::get()
                  .into_iter()
                  .find(|(duration, _)| *duration == stake.duration)
                  .map(|(_, reward)| reward)
                  .unwrap_or_else(Percent::zero)
                  * session_fee_for_currency;

                // calculate proportional reward base on the stake pool
                let staking_pool_percentage = Perquintill::from_rational(
                  stake.principal,
                  StakingPool::<T>::get(stake.currency_id).unwrap_or(0),
                );

                let proportional_reward = staking_pool_percentage * available_reward;
                stake.principal = stake.principal.saturating_add(proportional_reward);

                // add the `proportional_reward` to the staking pool
                StakingPool::<T>::mutate(stake.currency_id, |balance| {
                  if let Some(b) = balance {
                    *balance = Some(b.saturating_add(proportional_reward));
                  } else {
                    *balance = Some(proportional_reward);
                  }
                });

                // increment the distribution
                PendingStoredSessions::<T>::mutate(
                  *session_to_index,
                  |maybe_currently_rewarded: &mut Option<
                    BoundedVec<(CurrencyId, Balance), T::StakingRewardCap>,
                  >| {
                    match maybe_currently_rewarded {
                      Some(currently_rewarded) => {
                        let maybe_rewarded_for_current_currency = currently_rewarded
                          .iter_mut()
                          .find(|(currency_id, _)| *currency_id == stake.currency_id);
                        if let Some((_, balance)) = maybe_rewarded_for_current_currency {
                          *balance = balance.saturating_add(proportional_reward);
                        } else {
                          let _ =
                            currently_rewarded.try_push((stake.currency_id, proportional_reward));
                        }
                      }
                      None => {
                        *maybe_currently_rewarded = Some(
                          vec![(stake.currency_id, proportional_reward)]
                            .try_into()
                            .expect("too many rewards"),
                        )
                      }
                    }
                  },
                );

                // update the last session index for this stake
                stake.last_session_index_compound = *session_to_index;

                log!(
                  trace,
                  "Recomputed rewards for {:?} with new balance: {} (initial: {})",
                  account_id,
                  stake.principal,
                  stake.initial_balance,
                );
              },
            );
          });

        Ok(())
      })
    }

    fn do_session_drain(session_index: SessionIndex) -> DispatchResult {
      let pending_session =
        PendingStoredSessions::<T>::take(session_index).ok_or(Error::<T>::InvalidSession)?;

      let mut pool: Vec<(CurrencyId, Balance)> = Default::default();
      let mut operator: Vec<(CurrencyId, Balance)> = Default::default();

      for (currency_id, distributed_amount) in pending_session {
        let collected_amount = SessionTotalFees::<T>::take(session_index, currency_id);
        let operator_amount = collected_amount.saturating_sub(distributed_amount);
        pool.push((currency_id, distributed_amount));
        operator.push((currency_id, operator_amount));

        // transfer from staking pallet account
        // to operator account the remaining funds
        T::CurrencyTidefi::transfer(
          currency_id,
          &Self::account_id(),
          &Self::operator_account(),
          operator_amount,
          true,
        )
        .map_err(|_| Error::<T>::TransferFeesFailed)?;
      }

      Self::deposit_event(Event::<T>::SessionFinished {
        session_index,
        pool,
        operator,
      });

      Ok(())
    }

    #[inline]
    pub fn do_on_idle_compound(remaining_weight: Weight) -> Weight {
      // any weight that is unaccounted for
      let mut unaccounted_weight = Weight::from(0_u64);
      let next_batch_size = QueueCompound::<T>::count().min(T::BatchSize::get());

      // determine the number of accounts to check. This is based on both `ErasToCheckPerBlock`
      // and `remaining_weight` passed on to us from the runtime executive.
      let max_weight = |b| {
        <T as Config>::WeightInfo::on_idle_compound(b)
          .saturating_add(T::DbWeight::get().reads(next_batch_size.into()))
      };

      let accounts: BoundedVec<_, T::BatchSize> = QueueCompound::<T>::drain()
        .take(T::BatchSize::get() as usize)
        .map(|(a, _)| a)
        .collect::<Vec<_>>()
        .try_into()
        .expect("take ensures bound is met; qed");

      let pre_length = accounts.len() as u32;

      if max_weight(pre_length).gt(&remaining_weight) {
        log!(debug, "early exit because max weight is reached");
        return T::DbWeight::get()
          .reads(3)
          .saturating_add(unaccounted_weight);
      }

      unaccounted_weight
        .saturating_accrue(T::DbWeight::get().reads_writes(pre_length as u64, pre_length as u64));

      log!(
        debug,
        "checking {:?} accounts, next_batch_size = {:?}, remaining_weight = {:?}",
        pre_length,
        next_batch_size,
        remaining_weight,
      );

      let unchecked_sessions_to_check: Vec<SessionIndex> = PendingStoredSessions::<T>::iter()
        .map(|(key, _)| key)
        .collect();

      let collected_fees_by_session: Vec<(SessionIndex, Vec<(CurrencyId, Balance)>)> =
        unchecked_sessions_to_check
          .iter()
          .map(|session| {
            (
              *session,
              SessionTotalFees::<T>::iter_prefix(*session).collect(),
            )
          })
          .collect();

      // try to drain a session
      let do_drain_session = |session: SessionIndex| {
        let result = Self::do_session_drain(session);
        log!(info, "session drained {}, outcome: {:?}", session, result);
      };

      // try to compound an account id
      let do_compound = |account_id: &T::AccountId| {
        let result = Self::do_account_compound(account_id, &collected_fees_by_session);
        log!(
          info,
          "account compound {:?}, outcome: {:?}",
          account_id,
          result
        );
      };

      // if there is no accounts remaining, that mean we can drain the pending sessions
      if accounts.is_empty() {
        let size = unchecked_sessions_to_check.len() as u32;
        unchecked_sessions_to_check
          .iter()
          .for_each(|session| do_drain_session(*session));

        Self::deposit_event(Event::<T>::BatchFinished {
          size,
          kind: BatchType::Compound,
        });

        <T as Config>::WeightInfo::on_idle_compound_finalize(size)
          .saturating_add(unaccounted_weight)
      } else {
        accounts
          .iter()
          .for_each(|account_id| do_compound(account_id));

        Self::deposit_event(Event::<T>::BatchCompound { size: pre_length });

        <T as Config>::WeightInfo::on_idle_compound(pre_length).saturating_add(unaccounted_weight)
      }
    }

    #[inline]
    pub fn do_on_idle_unstake(remaining_weight: Weight) -> Weight {
      // any weight that is unaccounted for
      let mut unaccounted_weight = Weight::from(0_u64);

      let current_block = T::Security::get_current_block_count();
      let accounts = QueueUnstake::<T>::iter()
        .filter(|(_, (_, expiration))| *expiration <= current_block)
        .take(T::BatchSize::get() as usize)
        .collect::<Vec<(Hash, (T::AccountId, T::BlockNumber))>>();

      // clean queue
      accounts
        .iter()
        .for_each(|(stake_id, (_, _))| QueueUnstake::<T>::remove(stake_id));

      let next_batch_size = accounts.len() as u32;
      // determine the number of accounts to check. This is based on both `ErasToCheckPerBlock`
      // and `remaining_weight` passed on to us from the runtime executive.
      let max_weight = |b| {
        <T as Config>::WeightInfo::on_idle_unstake(b)
          .saturating_add(T::DbWeight::get().reads(next_batch_size.into()))
      };

      if max_weight(next_batch_size).gt(&remaining_weight) {
        log!(debug, "early exit because max weight is reached");
        return T::DbWeight::get()
          .reads(3)
          .saturating_add(unaccounted_weight);
      }

      unaccounted_weight.saturating_accrue(
        T::DbWeight::get().reads_writes(next_batch_size as u64, next_batch_size as u64),
      );

      log!(
        debug,
        "next_batch_size = {:?}, remaining_weight = {:?}",
        next_batch_size,
        remaining_weight,
      );

      let process_unstake = |account_id, hash| {
        let stake = AccountStakes::<T>::get(&account_id)
          .into_iter()
          .find(|s| s.unique_id == hash);
        if let Some(stake) = stake {
          let result = Self::do_process_unstake(&account_id, stake.unique_id);
          log!(info, "unstaked {:?}, outcome: {:?}", account_id, result);
        }
      };

      accounts
        .into_iter()
        .for_each(|(hash, (account_id, _))| process_unstake(account_id, hash));

      if next_batch_size > 0 {
        Self::deposit_event(Event::<T>::BatchFinished {
          size: next_batch_size,
          kind: BatchType::Unstake,
        });
      }

      <T as Config>::WeightInfo::on_idle_unstake(next_batch_size).saturating_add(unaccounted_weight)
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

    fn account_stakes_size() -> u64 {
      AccountStakes::<T>::count().into()
    }

    // triggered by `Fees` pallet when the session end
    fn on_session_end(
      session_index: SessionIndex,
      session_trade_values: Vec<(CurrencyId, Balance)>,
      fees_account_id: T::AccountId,
    ) -> Result<(), DispatchError> {
      let prepare_session = |currency_id: CurrencyId, balance: Balance| {
        let destination = if currency_id == CurrencyId::Tdfy {
          Self::operator_account()
        } else {
          Self::account_id()
        };

        // Transfer all fees collected by `Fees` pallet to `Staking` pallet for the redistribution.
        let result =
          T::CurrencyTidefi::transfer(currency_id, &fees_account_id, &destination, balance, false);
        log!(
          info,
          "session {} prepared for {:?}, outcome: {:?}",
          session_index,
          currency_id,
          result
        );

        if result.is_ok() && currency_id != CurrencyId::Tdfy {
          SessionTotalFees::<T>::insert(session_index, currency_id, balance);
        }

        result.is_ok()
      };

      // 1. Prepare session
      //   - If currency is TDFY, Operator take all the profits, as there is no staking pool
      //   - If currency is Wrapped, we transfer the fees from `fees_account_id` to Pallet account
      //   - If currency is Wrapped, we add the `SessionTotalFees` to the fees map for staking pool redistribution
      let sessions: Vec<(CurrencyId, Balance)> = session_trade_values
        .into_iter()
        .filter(|(currency_id, balance)| prepare_session(*currency_id, *balance))
        .collect();

      if !sessions.is_empty() {
        // 2. Mark the session index has last session finished
        InterestCompoundLastSession::<T>::put(session_index);
        // 3. Add the session into the queue (required for `on_idle`)
        PendingStoredSessions::<T>::insert::<
          u64,
          BoundedVec<(CurrencyId, Balance), T::StakingRewardCap>,
        >(session_index, Default::default());
        // 4. Grab all current account (keys) and add them into the `QueueCompound` who will be drained by `on_idle`
        AccountStakes::<T>::iter_keys()
          .for_each(|account| QueueCompound::<T>::insert(account, session_index));
      }

      Ok(())
    }
  }
}
