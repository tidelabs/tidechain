//! Benchmarking setup for pallet-tidefi
use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::fungibles::Mutate;
use frame_system::RawOrigin;
use tidefi_primitives::CurrencyId;

const INITIAL_AMOUNT: u128 = 500_000_000;
const IA_MULTIPLIER: u32 = 2;
const TEST_TOKEN: u32 = 2;

benchmarks! {
   stake {
      let caller: T::AccountId = whitelisted_caller();
      let balance = INITIAL_AMOUNT.saturating_mul(IA_MULTIPLIER.into());
      <T as Config>::CurrencyTidefi::mint_into(CurrencyId::Wrapped(TEST_TOKEN), &caller, balance as u128).expect("Unable to mint token");
   }: _(RawOrigin::Signed(caller), CurrencyId::Wrapped(TEST_TOKEN), INITIAL_AMOUNT, (14400_u32 * 15_u32).into())

   unstake {
      let caller: T::AccountId = whitelisted_caller();
      let balance = INITIAL_AMOUNT.saturating_mul(IA_MULTIPLIER.into());
      <T as Config>::CurrencyTidefi::mint_into(CurrencyId::Wrapped(TEST_TOKEN), &caller, balance as u128).expect("Unable to mint token");

      let stake_id = Pallet::<T>::add_account_stake(&caller, CurrencyId::Wrapped(TEST_TOKEN), INITIAL_AMOUNT, (14400_u32 * 15_u32).into()).expect("Unable to stake asset");
   }: _(RawOrigin::Signed(caller), stake_id, true)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
