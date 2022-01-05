//! Benchmarking setup for pallet-wrapr
use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{inherent::Vec, traits::fungibles::Mutate};
use frame_system::RawOrigin;
use tidefi_primitives::CurrencyId;
const INITIAL_AMOUNT: u128 = 500_000_000_000_000;
const IA_MULTIPLIER: u32 = 2;

benchmarks! {
   request_withdrawal {
      let caller: T::AccountId = whitelisted_caller();
      let balance = INITIAL_AMOUNT.saturating_mul(IA_MULTIPLIER.into());
      T::CurrencyWrapr::mint_into(CurrencyId::Wrapped(1), &caller, balance as u128).expect("Unable to mint token");
   }: _(RawOrigin::Signed(caller), CurrencyId::Wrapped(1), INITIAL_AMOUNT.into(), Vec::new())
   transfer {
      let caller: T::AccountId = whitelisted_caller();
   }: _(RawOrigin::Signed(caller.clone()), caller.clone(), CurrencyId::Tide, INITIAL_AMOUNT)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
