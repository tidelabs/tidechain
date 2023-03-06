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

//! Benchmarking setup for pallet-fees
use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_support::{
  assert_ok,
  traits::{fungibles::Mutate, Get, Hooks},
};
use frame_system::RawOrigin;
use scale_info::prelude::format;
use sp_runtime::traits::{Saturating, StaticLookup};
use sp_std::prelude::*;
use tidefi_primitives::{
  pallet::{FeesExt, StakingExt},
  CurrencyId, Fee,
};

const MAX_ASSET_ID_IN_PRIMITIVES: u32 = 6;
const MAX_ACCOUNTS_BENCH: u32 = 10_000;
const TEST_TOKEN: u32 = 2;
const USER_SEED: u32 = 0;
const INITIAL_AMOUNT: u128 = 500_000_000;
const USER_FEE: u128 = 1_000_000;
const IA_MULTIPLIER: u32 = 2;
const FIFTEEN_DAYS: u32 = 14400 * 15;

fn create_swap_fees_batch<T: Config>(batch_size: u32)
where
  T: pallet_assets::Config,
  <T as pallet_assets::Config>::AssetIdParameter: From<u32>,
  <T as pallet_assets::Config>::AssetId: From<u32>,
  <<T as frame_system::Config>::Lookup as StaticLookup>::Source:
    From<<T as frame_system::Config>::AccountId>,
  <T as pallet_assets::Config>::Balance: From<u128>,
{
  (0..batch_size)
    .map(|i| {
      if pallet_assets::Pallet::<T>::asset_details::<<T as pallet_assets::Config>::AssetId>(
        i.into(),
      )
      .is_none()
      {
        let account = frame_benchmarking::account::<T::AccountId>("asset_owner", i, USER_SEED);
        assert_ok!(pallet_assets::Pallet::<T>::force_create(
          RawOrigin::Root.into(),
          i.into(),
          account.into(),
          true,
          1.into(),
        ));

        assert_ok!(pallet_assets::Pallet::<T>::force_set_metadata(
          RawOrigin::Root.into(),
          i.into(),
          format!("TEST{}", i).into(),
          format!("TST{}", i).into(),
          12_u8,
          false,
        ));
      }

      SessionTotalFees::<T>::insert(
        1,
        CurrencyId::Wrapped(i),
        Fee {
          amount: USER_FEE.saturating_mul(100),
          fee: USER_FEE,
          fee_tdfy: 100_000_000_000,
        },
      );

      // fund fees pallet with the expected rewards
      assert_ok!(<T as Config>::CurrencyTidefi::mint_into(
        CurrencyId::Wrapped(i),
        &Pallet::<T>::account_id(),
        USER_FEE,
      ));
    })
    .collect()
}

fn create_stake_batch<T: Config>(batch_size: u32) -> Vec<T::AccountId>
where
  T: pallet_tidefi_stake::Config,
{
  (0..batch_size)
    .map(|i| {
      let account = frame_benchmarking::account::<T::AccountId>("stake", i, USER_SEED);
      fund_and_stake_account::<T>(&account);
      account
    })
    .collect()
}

fn fund_and_stake_account<T: Config>(account: &T::AccountId)
where
  T: pallet_tidefi_stake::Config,
{
  assert_ok!(<T as Config>::CurrencyTidefi::mint_into(
    CurrencyId::Wrapped(TEST_TOKEN),
    &account,
    INITIAL_AMOUNT.saturating_mul(IA_MULTIPLIER.into()),
  ));

  assert_ok!(pallet_tidefi_stake::Pallet::<T>::stake(
    RawOrigin::Signed(account.clone()).into(),
    CurrencyId::Wrapped(TEST_TOKEN),
    INITIAL_AMOUNT,
    FIFTEEN_DAYS.into()
  ));
}

benchmarks! {
  where_clause {
    where T: pallet_security::Config + pallet_tidefi_stake::Config + pallet_assets::Config,
      <T as pallet_assets::Config>::AssetIdParameter: From<u32>,
      <T as pallet_assets::Config>::AssetId: From<u32>,
      <<T as frame_system::Config>::Lookup as StaticLookup>::Source: From<<T as frame_system::Config>::AccountId>,
      <T as pallet_assets::Config>::Balance: From<u128>
  }

  on_finalize {
    let a in 1 .. MAX_ACCOUNTS_BENCH;
    // asset id available are 2-6
    let b in 1 .. MAX_ASSET_ID_IN_PRIMITIVES;

    // jump to block 1
    pallet_security::CurrentBlockCount::<T>::put::<T::BlockNumber>(1_u32.into());
    // start fees era
    Pallet::<T>::on_finalize(1_u32.into());
    let current_era = ActiveEra::<T>::get().expect("created before");
    assert_eq!(current_era.start_block, Some(1_u32.into()));
    assert_eq!(current_era.start_session_index, Some(1_u64));

    // create `AccountStakes` the size of `a`
    create_stake_batch::<T>(a);
    assert_eq!(T::Staking::account_stakes_size(), a as u64);

    // register swap fees for each currency
    create_swap_fees_batch::<T>(b);
    assert_eq!(SessionTotalFees::<T>::iter_prefix(CurrentSession::<T>::get()).count() as u32, b);

    // jump to expected end of session
    let expected_block = T::BlocksPerSession::get().saturating_add(1_u32.into());
    pallet_security::CurrentBlockCount::<T>::put(expected_block);
  }
  : {
    Pallet::<T>::on_finalize(expected_block);
  }
  verify {
    assert_eq!(
      pallet_tidefi_stake::QueueCompound::<T>::count(),
      a
    )
  }

}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
