//! Benchmarking setup for pallet-wrapr

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{self, RawOrigin};
use tidefi_primitives::{AccountId, AssetId};

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
  frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

const INITIAL_AMOUNT: u32 = 500_000_000;
const TIDE: AssetId = 0;
const ETH: AssetId = 1;

benchmarks! {
   // benchmark quorum status
   set_status {}: _(RawOrigin::Root, true)
   confirm_withdrawal {
      let request_id = 1;
   }: _(RawOrigin::Root, request_id)
   mint {
      let caller: T::AccountId = whitelisted_caller();
   }: _(RawOrigin::Root, caller, TIDE.into(), INITIAL_AMOUNT.into())

}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
