//! Benchmarking setup for pallet-wrapr

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{self, RawOrigin};
use tidefi_primitives::{CurrencyId, Hash};

const SEED: u32 = 0;
const ADMIN_ID: u32 = 1;
const INITIAL_AMOUNT: u32 = 500_000_000;

fn _assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
  frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn pre_set_auth<T: Config>() -> T::AccountId {
  let user: T::AccountId = account("admin", ADMIN_ID, SEED);
  QuorumAccountId::<T>::put(user.clone());
  user
}

benchmarks! {
   set_status {
      let user = pre_set_auth::<T>();
   }: _(RawOrigin::Signed(user), true)
   confirm_withdrawal {
      let request_id = Hash::zero();
      let user = pre_set_auth::<T>();
   }: _(RawOrigin::Signed(user), request_id)
   mint {
      let caller: T::AccountId = whitelisted_caller();
      let user = pre_set_auth::<T>();
   }: _(RawOrigin::Signed(user), caller, CurrencyId::Tide, INITIAL_AMOUNT.into())
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
