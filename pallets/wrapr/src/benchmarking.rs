//! Benchmarking setup for pallet-wrapr
use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use tidefi_primitives::CurrencyId;

const INITIAL_AMOUNT: u32 = 500_000_000;

benchmarks! {
   request_withdrawal {
      let caller: T::AccountId = whitelisted_caller();
   }: _(RawOrigin::Signed(caller), CurrencyId::Tide, INITIAL_AMOUNT.into(), Vec::new())
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
