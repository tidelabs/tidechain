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

use crate::{Call, Config, Pallet, VestingSchedule, VESTING_LOCK_ID, *};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::WithdrawReasons;
use frame_system::RawOrigin;
use sp_runtime::traits::Bounded;

type BalanceOf<T> =
  <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

benchmarks! {
   stop_vesting_schedules {
      let account: T::AccountId = whitelisted_caller();
      let owner = T::Lookup::unlookup(account.clone());

      T::Currency::make_free_balance_be(&account, BalanceOf::<T>::max_value());
      let reasons = WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE;
      T::Currency::set_lock(VESTING_LOCK_ID, &account, 20u32.into(), reasons);
      VestingSchedules::<T>::try_append(account,
         VestingSchedule {
            start: 0u64,
            period: 10u64,
            period_count: 2u32,
            per_period: 10u128,
         }
      );

      frame_system::Pallet::<T>::set_block_number(
         frame_system::Pallet::<T>::block_number() + T::BlockNumber::from(11_u32),
      );
   }: _(RawOrigin::Root, owner)
}

impl_benchmark_test_suite!(
  Pallet,
  crate::mock::ExtBuilder::build(),
  crate::mock::Runtime
);
