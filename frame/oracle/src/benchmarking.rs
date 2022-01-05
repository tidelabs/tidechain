//! Benchmarking setup for pallet-wrapr

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{self, RawOrigin};
use sp_std::vec::Vec;
use tidefi_primitives::Hash;

const SEED: u32 = 0;
const ADMIN_ID: u32 = 1;

fn _assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
  frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn pre_set_auth<T: Config>() -> T::AccountId {
  let user: T::AccountId = account("admin", ADMIN_ID, SEED);
  OracleAccountId::<T>::put(user.clone());
  OracleStatus::<T>::put(true);
  user
}

benchmarks! {
   set_status {
      let user = pre_set_auth::<T>();
   }: _(RawOrigin::Signed(user), true)
   set_account_id {
      let user = pre_set_auth::<T>();
      let caller: T::AccountId = whitelisted_caller();
   }: _(RawOrigin::Signed(user), caller)
   confirm_trade {
      let user = pre_set_auth::<T>();
      let request_id = Hash::zero();
   }: _(RawOrigin::Signed(user), request_id, Vec::new())
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
