//! Benchmarking setup for pallet-wrapr
use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::inherent::Vec;
use frame_system::RawOrigin;
use tidefi_primitives::CurrencyId;

const INITIAL_AMOUNT: u32 = 500_000_000;

benchmarks! {
   request_withdrawal {
      let caller: T::AccountId = whitelisted_caller();
   }: _(RawOrigin::Signed(caller), CurrencyId::Tide, INITIAL_AMOUNT.into(), Vec::new())
   transfer {
      let caller: T::AccountId = whitelisted_caller();
   }: _(RawOrigin::Signed(caller.clone()), caller.clone(), CurrencyId::Tide, INITIAL_AMOUNT.into())
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
