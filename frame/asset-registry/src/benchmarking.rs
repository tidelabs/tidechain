#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_system::{self, RawOrigin};
use tidefi_primitives::CurrencyId;

const SEED: u32 = 0;
const ADMIN_ID: u32 = 1;

fn _assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
  frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn pre_set_auth<T: Config>() -> T::AccountId {
  let user: T::AccountId = account("admin", ADMIN_ID, SEED);
  AssetRegistryAccountId::<T>::put(user.clone());
  user
}

benchmarks! {
   set_status {
    let user = pre_set_auth::<T>();
   }: _(RawOrigin::Signed(user), CurrencyId::Tide, true)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
