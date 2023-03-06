// Copyright 2021-2022 Semantic Network Ltd.
// This file is part of Tidechain.

// Tidechain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tidechain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tidechain.  If not, see <http://www.gnu.org/licenses/>.

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller};
use frame_support::traits::fungibles::Mutate;
use frame_system::{self, RawOrigin};
use sp_runtime::{traits::StaticLookup, Permill};
use tidefi_primitives::{pallet::OracleExt, CurrencyId, SwapConfirmation, SwapType};

const SEED: u32 = 0;
const ADMIN_ID: u32 = 1;
const USER_ID: u32 = 2;
const MM_ID: u32 = 3;

const TEST_TOKEN: u32 = 2;
const TEST_TOKEN2: u32 = 3;

fn _assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
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
   add_market_maker {
      let user = pre_set_auth::<T>();
      let caller: T::AccountId = whitelisted_caller();
   }: _(RawOrigin::Signed(user), caller)
   update_assets_value {
      let user = pre_set_auth::<T>();
   }: _(RawOrigin::Signed(user), Default::default())
   remove_market_maker {
      let user = pre_set_auth::<T>();
      let caller: T::AccountId = whitelisted_caller();
      MarketMakers::<T>::insert(caller.clone(), true);
   }: _(RawOrigin::Signed(user), caller)
   confirm_swap {
      let user = pre_set_auth::<T>();
      let account_id: T::AccountId = account("user", USER_ID, SEED);
      let mm_account_id: T::AccountId = account("mm", MM_ID, SEED);
      let caller_lookup = T::Lookup::unlookup(user.clone());

      // mint tokens
      T::CurrencyTidefi::mint_into(CurrencyId::Wrapped(TEST_TOKEN), &account_id, 2_000_000_000_000).expect("Unable to mint token");
      T::CurrencyTidefi::mint_into(CurrencyId::Wrapped(TEST_TOKEN2), &mm_account_id, 2_000_000_000_000).expect("Unable to mint token");

      // create requests
      let user_request = Pallet::<T>::add_new_swap_in_queue(account_id,
         CurrencyId::Wrapped(TEST_TOKEN),
         1_000_000_000_000,
         CurrencyId::Wrapped(TEST_TOKEN2),
         1_000_000_000_000,
         <frame_system::Pallet<T>>::block_number(),
         [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
         ],
         false,
         SwapType::Limit,
         Permill::from_percent(2),
      ).unwrap();

      let mm_request = Pallet::<T>::add_new_swap_in_queue(mm_account_id,
         CurrencyId::Wrapped(TEST_TOKEN2),
         1_000_000_000_000,
         CurrencyId::Wrapped(TEST_TOKEN),
         1_000_000_000_000,
         <frame_system::Pallet<T>>::block_number(),
         [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
         ],
         false,
         SwapType::Limit,
         Permill::from_percent(2),
      ).unwrap();

   }: _(RawOrigin::Signed(user), user_request.0, vec![SwapConfirmation { request_id: mm_request.0, amount_to_receive: 1_000_000_000_000, amount_to_send: 1_000_000_000_000 }])
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
