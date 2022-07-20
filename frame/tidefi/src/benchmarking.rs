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

use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
  inherent::Vec,
  traits::{fungibles::Mutate, Hooks},
};
use frame_system::RawOrigin;
use tidefi_primitives::{pallet::SunriseExt, CurrencyId, SwapType};
const INITIAL_AMOUNT: u128 = 500_000_000_000_000;
const IA_MULTIPLIER: u32 = 2;
const TEST_TOKEN: u32 = 2;

benchmarks! {
   where_clause { where T: pallet_fees::Config + pallet_sunrise::Config + pallet_security::Config }

   withdrawal {
      let caller: T::AccountId = whitelisted_caller();
      let balance = INITIAL_AMOUNT.saturating_mul(IA_MULTIPLIER.into());
      <T as Config>::CurrencyTidefi::mint_into(CurrencyId::Wrapped(TEST_TOKEN), &caller, balance as u128).expect("Unable to mint token");
   }: _(RawOrigin::Signed(caller), CurrencyId::Wrapped(TEST_TOKEN), INITIAL_AMOUNT.into(), Vec::new())
   swap {
      let caller: T::AccountId = whitelisted_caller();
      let balance = INITIAL_AMOUNT.saturating_mul(IA_MULTIPLIER.into());
      <T as Config>::CurrencyTidefi::mint_into(CurrencyId::Wrapped(TEST_TOKEN), &caller, balance as u128).expect("Unable to mint token");
      <T as Config>::CurrencyTidefi::mint_into(CurrencyId::Tdfy, &caller, balance as u128).expect("Unable to mint token");
   }: _(RawOrigin::Signed(caller), CurrencyId::Tdfy, INITIAL_AMOUNT.into(), CurrencyId::Wrapped(TEST_TOKEN), INITIAL_AMOUNT.into(), SwapType::Limit, None)
   transfer {
      let caller: T::AccountId = whitelisted_caller();
   }: _(RawOrigin::Signed(caller.clone()), caller.clone(), CurrencyId::Tdfy, INITIAL_AMOUNT)
   claim_sunrise_rewards {
      let caller: T::AccountId = whitelisted_caller();
      // init first era
      pallet_fees::Pallet::<T>::start_era();
      pallet_fees::Pallet::<T>::on_finalize(T::BlockNumber::from(1_u32));

      <T as Config>::CurrencyTidefi::mint_into(CurrencyId::Tdfy, &<T as Config>::Sunrise::account_id(), 1_000_000_000_000_000_000).unwrap();
      <pallet_sunrise::Rewards<T>>::insert(&caller, 1, 1_000_000_000_000);

      frame_system::Pallet::<T>::on_finalize(frame_system::Pallet::<T>::block_number());
      frame_system::Pallet::<T>::set_block_number(
        frame_system::Pallet::<T>::block_number() + T::BlockNumber::from(100_000_u32),
      );
      frame_system::Pallet::<T>::on_initialize(frame_system::Pallet::<T>::block_number());
      frame_system::Pallet::<T>::on_finalize(frame_system::Pallet::<T>::block_number());
      pallet_security::CurrentBlockCount::<T>::put(frame_system::Pallet::<T>::block_number());

   }: _(RawOrigin::Signed(caller), 1)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
