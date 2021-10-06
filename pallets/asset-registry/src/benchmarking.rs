//! Benchmarking setup for pallet-wrapr

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::{self, RawOrigin};
use tidefi_primitives::CurrencyId;

fn _assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
  frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
   set_status {}: _(RawOrigin::Root, CurrencyId::Tide, true)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
