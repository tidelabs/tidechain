//! Benchmarking setup for pallet-tidefi

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller};
use frame_support::traits::fungibles::Mutate;
use frame_system::{self, RawOrigin};
use sp_runtime::traits::StaticLookup;
use tidefi_primitives::{pallet::OracleExt, CurrencyId, SwapConfirmation};

const SEED: u32 = 0;
const ADMIN_ID: u32 = 1;
const USER_ID: u32 = 2;
const MM_ID: u32 = 3;

fn _assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
  frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn pre_set_auth<T: Config>() -> T::AccountId {
  let user: T::AccountId = account("admin", ADMIN_ID, SEED);
  OracleAccountId::<T>::put(user.clone());
  OracleStatus::<T>::put(true);
  user
}

benchmarks! {
   set_status {
      let user = pre_set_auth::<T>();
   }: _(RawOrigin::Signed(user), true)
   set_account_id {
      let user = pre_set_auth::<T>();
      let caller: T::AccountId = whitelisted_caller();
   }: _(RawOrigin::Signed(user), caller)
   confirm_swap {
      let user = pre_set_auth::<T>();
      let account_id: T::AccountId = account("user", USER_ID, SEED);
      let mm_account_id: T::AccountId = account("mm", MM_ID, SEED);
      let caller_lookup = T::Lookup::unlookup(user.clone());

      // create assets
      pallet_assets::Pallet::<T>::force_create(RawOrigin::Root.into(), 1, caller_lookup.clone(), true, 1).expect("Unable to create assets");
      pallet_assets::Pallet::<T>::force_set_metadata(RawOrigin::Root.into(), 1, "Test".into(), "TST".into(), 6, false).expect("Unable to update assets");
      pallet_assets::Pallet::<T>::force_create(RawOrigin::Root.into(), 2, caller_lookup, true, 1).expect("Unable to create assets");
      pallet_assets::Pallet::<T>::force_set_metadata(RawOrigin::Root.into(), 2, "Test2".into(), "TST2".into(), 6, false).expect("Unable to update assets");

      // mint tokens
      T::CurrencyTidefi::mint_into(CurrencyId::Wrapped(1), &account_id, 2_000_000_000_000).expect("Unable to mint token");
      T::CurrencyTidefi::mint_into(CurrencyId::Wrapped(2), &mm_account_id, 2_000_000_000_000).expect("Unable to mint token");

      // create requests
      let user_request = Pallet::<T>::add_new_swap_in_queue(account_id,
         CurrencyId::Wrapped(1),
         1_000_000_000_000,
         CurrencyId::Wrapped(2),
         1_000_000_000_000,
         <frame_system::Pallet<T>>::block_number(),
         [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
         ]
      ).unwrap();

      let mm_request = Pallet::<T>::add_new_swap_in_queue(mm_account_id,
         CurrencyId::Wrapped(2),
         1_000_000_000_000,
         CurrencyId::Wrapped(1),
         1_000_000_000_000,
         <frame_system::Pallet<T>>::block_number(),
         [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
         ]
      ).unwrap();

   }: _(RawOrigin::Signed(user), user_request.0, vec![SwapConfirmation { request_id: mm_request.0, amount_to_receive: 1_000_000_000_000, amount_to_send: 1_000_000_000_000 }])
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
