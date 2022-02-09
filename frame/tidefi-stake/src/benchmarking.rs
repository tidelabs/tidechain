//! Benchmarking setup for pallet-tidefi
use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::fungibles::Mutate;
use frame_system::RawOrigin;
use tidefi_primitives::CurrencyId;

const INITIAL_AMOUNT: u128 = 500_000_000;
const IA_MULTIPLIER: u32 = 2;
const TEST_TOKEN: u32 = 4294967295;

benchmarks! {
   stake {
      let caller: T::AccountId = whitelisted_caller();
      let balance = INITIAL_AMOUNT.saturating_mul(IA_MULTIPLIER.into());
      <T as Config>::CurrencyTidefi::mint_into(CurrencyId::Wrapped(TEST_TOKEN), &caller, balance as u128).expect("Unable to mint token");
   }: _(RawOrigin::Signed(caller), CurrencyId::Wrapped(TEST_TOKEN), INITIAL_AMOUNT, (14400_u32 * 15_u32).into())
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
