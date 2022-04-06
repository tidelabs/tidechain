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
      T::CurrencyTidefi::mint_into(CurrencyId::Tifi, &caller, balance as u128).expect("Unable to mint token");
   }: _(RawOrigin::Signed(caller), CurrencyId::Tifi, INITIAL_AMOUNT.into(), CurrencyId::Wrapped(TEST_TOKEN), INITIAL_AMOUNT.into(), SwapType::Limit, None)
   transfer {
      let caller: T::AccountId = whitelisted_caller();
   }: _(RawOrigin::Signed(caller.clone()), caller.clone(), CurrencyId::Tifi, INITIAL_AMOUNT)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
