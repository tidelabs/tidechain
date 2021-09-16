//! Benchmarking setup for pallet-wrapr

use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;

benchmarks! {
   // benchmark quorum status
   set_quorum_status {}: _(RawOrigin::Root, true)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
