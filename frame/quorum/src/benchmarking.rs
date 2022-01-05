//! Benchmarking setup for pallet-wrapr

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::fungibles::Mutate;
use frame_system::{self, RawOrigin};
use sp_runtime::traits::StaticLookup;
use tidefi_primitives::{pallet::QuorumExt, CurrencyId};

const SEED: u32 = 0;
const ADMIN_ID: u32 = 1;
const USER_ID: u32 = 2;

fn _assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
  frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn pre_set_auth<T: Config>() -> T::AccountId {
  let user: T::AccountId = account("admin", ADMIN_ID, SEED);
  QuorumAccountId::<T>::put(user.clone());
  QuorumStatus::<T>::put(true);
  user
}

benchmarks! {
   set_status {
      let user = pre_set_auth::<T>();
   }: _(RawOrigin::Signed(user), true)
   confirm_withdrawal {
      let user = pre_set_auth::<T>();
      let account_id: T::AccountId = account("user", USER_ID, SEED);
      let user_lookup = T::Lookup::unlookup(account_id.clone());
      let caller_lookup = T::Lookup::unlookup(user.clone());

      pallet_assets::Pallet::<T>::force_create(RawOrigin::Root.into(), 1, caller_lookup.clone(), true, 1).expect("Unable to create assets");
      pallet_assets::Pallet::<T>::force_set_metadata(RawOrigin::Root.into(), 1, "Test".into(), "TST".into(), 6, false).expect("Unable to update assets");
      T::CurrencyWrapr::mint_into(CurrencyId::Wrapped(1), &account_id, 3_000_000_000_000).expect("Unable to mint token");
      let request = Pallet::<T>::add_new_withdrawal_in_queue(account_id, CurrencyId::Wrapped(1), 1_000_000_000_000, "0x9126fFd76a7e02B875326D5C5b4EDFfc20C7B553".into());

   }: _(RawOrigin::Signed(user), request.0)
   mint {
      let caller: T::AccountId = whitelisted_caller();
      let user = pre_set_auth::<T>();
   }: _(RawOrigin::Signed(user), caller, CurrencyId::Tide, 1_000_000_000_000_000)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
