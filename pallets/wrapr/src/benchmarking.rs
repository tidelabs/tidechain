//! Benchmarking setup for pallet-wrapr

use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
   // benchmark quorum status
   set_quorum_status {}: _(RawOrigin::Root, true)
}

impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test,);
