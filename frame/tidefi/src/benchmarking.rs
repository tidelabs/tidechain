//! Benchmarking setup for pallet-tidefi
use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{inherent::Vec, traits::fungibles::Mutate};
use frame_system::RawOrigin;
use tidefi_primitives::{CurrencyId, SwapType};
const INITIAL_AMOUNT: u128 = 500_000_000_000_000;
const IA_MULTIPLIER: u32 = 2;
const TEST_TOKEN: u32 = 2;

benchmarks! {
   withdrawal {
      let caller: T::AccountId = whitelisted_caller();
      let balance = INITIAL_AMOUNT.saturating_mul(IA_MULTIPLIER.into());
      T::CurrencyTidefi::mint_into(CurrencyId::Wrapped(TEST_TOKEN), &caller, balance as u128).expect("Unable to mint token");
   }: _(RawOrigin::Signed(caller), CurrencyId::Wrapped(TEST_TOKEN), INITIAL_AMOUNT.into(), Vec::new())
   swap {
      let caller: T::AccountId = whitelisted_caller();
      let balance = INITIAL_AMOUNT.saturating_mul(IA_MULTIPLIER.into());
      T::CurrencyTidefi::mint_into(CurrencyId::Wrapped(TEST_TOKEN), &caller, balance as u128).expect("Unable to mint token");
      T::CurrencyTidefi::mint_into(CurrencyId::Tide, &caller, balance as u128).expect("Unable to mint token");
   }: _(RawOrigin::Signed(caller), CurrencyId::Tide, INITIAL_AMOUNT.into(), CurrencyId::Wrapped(TEST_TOKEN), INITIAL_AMOUNT.into(), SwapType::Limit, None)
   transfer {
      let caller: T::AccountId = whitelisted_caller();
   }: _(RawOrigin::Signed(caller.clone()), caller.clone(), CurrencyId::Tide, INITIAL_AMOUNT)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
