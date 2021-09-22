//! Benchmarking setup for pallet-wrapr

use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use sp_runtime::{traits::AtLeast32BitUnsigned, FixedPointOperand};
use tidefi_primitives::AssetId;

const INITIAL_AMOUNT: u32 = 500_000_000;
const TIDE: AssetId = 0;
const ETH: AssetId = 1;

benchmarks! {
   where_clause {
      where
          BalanceOf<T>: FixedPointOperand,
          AssetIdOf<T>: AtLeast32BitUnsigned,
   }

   request_withdrawal {
      let caller: T::AccountId = whitelisted_caller();
   }: _(RawOrigin::Signed(caller), TIDE.into(), INITIAL_AMOUNT.into())
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
