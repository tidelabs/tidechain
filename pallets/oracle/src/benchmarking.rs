//! Benchmarking setup for pallet-wrapr

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{self, RawOrigin};
use tidefi_primitives::CurrencyId;

fn _assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
  frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

const INITIAL_AMOUNT: u32 = 500_000_000;

benchmarks! {
   set_status {}: _(RawOrigin::Root, true)
   set_account_id {}: _(RawOrigin::Root, true)
   confirm_trade {
      let request_id = 1;
   }: _(RawOrigin::Root, request_id)

}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
