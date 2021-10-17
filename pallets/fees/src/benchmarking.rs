//! Benchmarking setup for pallet-wrapr
use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
//use frame_support::inherent::Vec;
use frame_system::RawOrigin;
use tidefi_primitives::CurrencyId;

const INITIAL_AMOUNT: u32 = 500_000_000;

benchmarks! {
   stake {
      let caller: T::AccountId = whitelisted_caller();
   }: _(RawOrigin::Signed(caller), CurrencyId::Tide, INITIAL_AMOUNT.into(), 1)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
