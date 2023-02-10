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

//! Benchmarking setup for pallet-tidefi
use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
  assert_ok,
  traits::{fungibles::Mutate, Get, OnIdle},
};
use frame_system::RawOrigin;
use sp_runtime::traits::{Saturating, Zero};
use sp_std::prelude::*;
use tidefi_primitives::{pallet::StakingExt, CurrencyId, SessionIndex};

const USER_SEED: u32 = 0;
const INITIAL_AMOUNT: u128 = 500_000_000;
const IA_MULTIPLIER: u32 = 2;
const TEST_TOKEN: u32 = 2;
const FIFTEEN_DAYS: u32 = 14400 * 15;

fn assert_event<T: Config>(generic_event: <T as Config>::Event) {
  frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

fn on_idle_full_block<T: Config>() {
  let remaining_weight = <T as frame_system::Config>::BlockWeights::get().max_block;
  Pallet::<T>::on_idle(Zero::zero(), remaining_weight);
}

fn create_stake_batch<T: Config>(batch_size: u32) -> Vec<T::AccountId> {
  (0..batch_size)
    .map(|i| {
      let account = frame_benchmarking::account::<T::AccountId>("uncompound_staker", i, USER_SEED);
      fund_and_stake_account::<T>(&account);
      account
    })
    .collect()
}

fn create_session_batch<T: Config>(batch_size: u32) {
  (0..batch_size).for_each(|i| trigger_on_session_end::<T>(i.into()))
}

fn fund_and_stake_account<T: Config>(account: &T::AccountId) {
  assert_ok!(<T as Config>::CurrencyTidefi::mint_into(
    CurrencyId::Wrapped(TEST_TOKEN),
    account,
    INITIAL_AMOUNT.saturating_mul(IA_MULTIPLIER.into()),
  ));

  assert_ok!(Pallet::<T>::stake(
    RawOrigin::Signed(account.clone()).into(),
    CurrencyId::Wrapped(TEST_TOKEN),
    INITIAL_AMOUNT,
    FIFTEEN_DAYS.into()
  ));
}

fn trigger_on_session_end<T: Config>(session_index: SessionIndex) {
  let dummy_fee_account = frame_benchmarking::account::<T::AccountId>("fees_pallet", 1, USER_SEED);
  assert_ok!(<T as Config>::CurrencyTidefi::mint_into(
    CurrencyId::Wrapped(TEST_TOKEN),
    &dummy_fee_account,
    INITIAL_AMOUNT.saturating_mul(100),
  ));
  assert_ok!(Pallet::<T>::on_session_end(
    session_index,
    vec![(
      CurrencyId::Wrapped(TEST_TOKEN),
      INITIAL_AMOUNT.saturating_mul(100),
    )],
    dummy_fee_account,
  ));
}

benchmarks! {
  where_clause { where T: pallet_security::Config }

  stake {
      let caller: T::AccountId = whitelisted_caller();
      fund_and_stake_account::<T>(&caller);
   }: _(RawOrigin::Signed(caller.clone()), CurrencyId::Wrapped(TEST_TOKEN), INITIAL_AMOUNT, FIFTEEN_DAYS.into())

   unstake {
      let caller: T::AccountId = whitelisted_caller();
      let balance = INITIAL_AMOUNT.saturating_mul(IA_MULTIPLIER.into());
      <T as Config>::CurrencyTidefi::mint_into(CurrencyId::Wrapped(TEST_TOKEN), &caller, balance as u128).expect("Unable to mint token");

      let stake_id = Pallet::<T>::add_account_stake(&caller, CurrencyId::Wrapped(TEST_TOKEN), INITIAL_AMOUNT, FIFTEEN_DAYS.into()).expect("Unable to stake asset");
   }: _(RawOrigin::Signed(caller), stake_id, true)

   on_idle_compound_finalize {
    let a = 1;
    let b in 1 .. T::BatchSize::get();

    create_stake_batch::<T>(a);
    create_session_batch::<T>(b);

    assert_eq!(QueueCompound::<T>::count(), a);
    assert_eq!(PendingStoredSessions::<T>::count(), b);

    on_idle_full_block::<T>();
    assert_eq!(QueueCompound::<T>::count(), 0);
  }
  : {
    on_idle_full_block::<T>();
  }
  verify {
   assert_event::<T>(Event::<T>::BatchFinished { size: b, kind: BatchType::Compound }.into());
   assert_eq!(PendingStoredSessions::<T>::count(), 0);
  }

  on_idle_compound {
    let b in 1 .. T::BatchSize::get();

    create_stake_batch::<T>(b);
    trigger_on_session_end::<T>(1);

    assert_eq!(QueueCompound::<T>::count(), b);
  }
  : {
    on_idle_full_block::<T>();
  }
  verify {
   assert_event::<T>(Event::<T>::BatchCompound { size: b }.into());
   assert_eq!(QueueCompound::<T>::count(), 0);
   assert_eq!(PendingStoredSessions::<T>::count(), 1);
  }

  on_idle_unstake {
    let b in 1 .. T::BatchSize::get();

    create_stake_batch::<T>(b).iter().for_each(|account_id| {
      let request_id = AccountStakes::<T>::get(account_id)
      .first()
      .expect("created previously")
      .unique_id;

      assert_ok!(Pallet::<T>::unstake(RawOrigin::Signed(account_id.clone()).into(), request_id, true));
    });

    assert_eq!(QueueUnstake::<T>::count(), b);
    let (_, expected_end_block)  = QueueUnstake::<T>::iter_values().next().unwrap();
    pallet_security::CurrentBlockCount::<T>::put(expected_end_block.saturating_add(1_u32.into()));
  }
  : {
    on_idle_full_block::<T>();
  }
  verify {
   assert_event::<T>(Event::<T>::BatchFinished { size: b, kind: BatchType::Unstake }.into());
   assert_eq!(QueueCompound::<T>::count(), 0);
  }
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
