//! Benchmarking setup for pallet-wrapr

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{self, RawOrigin};
use sp_std::vec::Vec;
use tidefi_primitives::Hash;

fn _assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
  frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
