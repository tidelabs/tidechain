#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::{benchmarks, benchmarks_instance_pallet};

use crate::Pallet as Securities;

benchmarks_instance_pallet! {
   set_status {
      let new_status = StatusCode::Maintenance;
   }: _(RawOrigin::Root, new_status)
   verify {
      assert_eq!(Security::status(), new_status);
   }

   impl_benchmark_test_suite!(
      Pallet,
      crate::tests_composite::ExtBuilder::default().build(),
      crate::tests_composite::Test,
   )
}
