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

use crate::{
  mock::{new_test_ext, AccountId, Adapter, Sunrise, Test},
  Error,
};
use frame_support::{assert_noop, assert_ok, traits::fungibles::Mutate};
use pallet_balances::Error as BalancesError;
use sp_runtime::{traits::CheckedDiv, FixedPointNumber, FixedU128};
use tidefi_primitives::{assets::Asset, pallet::SunriseExt, CurrencyId, Fee, OnboardingRebates};

const ALICE_ACCOUNT_ID: AccountId = AccountId(1);

#[test]
pub fn test_select_first_eligible_sunrise_pool() {
  new_test_ext().execute_with(|| {
    assert_eq!(Sunrise::sunrise_pools().len(), 2);

    // 0.001815 BTC for 1 TDFY
    let oracle_value = 550_964_187_327_800_u128;
    assert_ok!(Sunrise::register_exchange_rate(vec![(2, oracle_value)]));

    // 1 BTC should equal value set
    assert_eq!(
      Sunrise::try_get_tdfy_value(CurrencyId::Wrapped(2), Asset::Bitcoin.saturating_mul(1))
        .unwrap(),
      oracle_value.into()
    );

    // should have selected second pool
    assert_eq!(
      Sunrise::try_select_first_eligible_sunrise_pool(
        &Fee {
          amount: 1_000_000_000,
          fee: 10_000_000,
          fee_tdfy: Sunrise::try_get_tdfy_value(CurrencyId::Wrapped(2), 10_000_000).unwrap()
        },
        CurrencyId::Wrapped(2)
      )
      .unwrap()
      .unwrap()
      .id,
      2
    );

    // should have selected first pool
    assert_eq!(
      Sunrise::try_select_first_eligible_sunrise_pool(
        &Fee {
          amount: 100_000_000,
          fee: 1_000_000,
          fee_tdfy: Sunrise::try_get_tdfy_value(CurrencyId::Wrapped(2), 1_000_000).unwrap()
        },
        CurrencyId::Wrapped(2)
      )
      .unwrap()
      .unwrap()
      .id,
      1
    );
  })
}

#[test]
pub fn test_convert_fixed_balance_to_tdfy_balance() {
  // 0.03133 ETH for 1 TDFY
  let eth_exchange_rate = FixedU128::saturating_from_rational(
    31_330_000_000_000_000_u128,
    Asset::Ethereum.saturating_mul(1_u128),
  );
  assert_eq!(eth_exchange_rate.to_float(), 0.03133);
  let total_tdfy_for_one_eth = FixedU128::from(1_u128)
    .checked_div(&eth_exchange_rate)
    .unwrap();
  assert_eq!(total_tdfy_for_one_eth.to_float(), 31.91828917969997);

  assert_eq!(
    Sunrise::convert_fixed_balance_to_tdfy_balance(total_tdfy_for_one_eth).unwrap(),
    31_918_289_179_699
  );

  // 33.650000 USDT for 1 TDFY
  let ustd_exchange_rate =
    FixedU128::saturating_from_rational(33_650_000_u128, Asset::Tether.saturating_mul(1_u128));
  assert_eq!(ustd_exchange_rate.to_float(), 33.65);

  let total_tdfy_for_one_usdt = FixedU128::from(1_u128)
    .checked_div(&ustd_exchange_rate)
    .unwrap();
  assert_eq!(total_tdfy_for_one_usdt.to_float(), 0.029717682020802376);

  assert_eq!(
    Sunrise::convert_fixed_balance_to_tdfy_balance(total_tdfy_for_one_usdt).unwrap(),
    29_717_682_020
  );
}

#[test]
pub fn test_wrapped_asset_values() {
  new_test_ext().execute_with(|| {
    // 100k TDFY = 1 BTC
    let oracle_value = 100_000_000_000_000_000_u128;
    assert_ok!(Sunrise::register_exchange_rate(vec![(2, oracle_value)]));

    assert_eq!(
      Sunrise::try_get_tdfy_value(CurrencyId::Wrapped(2), 100_000_000).unwrap(),
      oracle_value.into()
    );

    assert_eq!(
      Sunrise::try_get_tdfy_value(CurrencyId::Wrapped(2), 10_000_000).unwrap(),
      10_000_000_000_000_000_u128.into()
    );

    assert_eq!(
      Sunrise::try_get_tdfy_value(CurrencyId::Wrapped(2), 625_000).unwrap(),
      625_000_000_000_000_u128.into()
    );
  });
}

#[test]
pub fn test_get_onboarding_rebares() {
  new_test_ext().execute_with(|| {
    let onboarding_rebate = Sunrise::onboarding();
    assert_eq!(
      Sunrise::get_next_onboarding_rebates(10_000_000_000_000_000_u128, &onboarding_rebate)
        .unwrap(),
      10_000_000_000_000_000_u128.into()
    );

    let onboarding_rebate = OnboardingRebates {
      // 50%
      available_amount: 24_000_000_000_000_000_000_u128,
      initial_amount: 48_000_000_000_000_000_000_u128,
    };

    assert_eq!(
      Sunrise::get_next_onboarding_rebates(10_000_000_000_000_000_u128, &onboarding_rebate)
        .unwrap(),
      5_000_000_000_000_000_u128.into()
    );

    let onboarding_rebate = OnboardingRebates {
      // 4.1666666666666666%
      available_amount: 2_000_000_000_000_000_000_u128,
      initial_amount: 48_000_000_000_000_000_000_u128,
    };

    // 416.6666666667
    assert_eq!(
      Sunrise::get_next_onboarding_rebates(10_000_000_000_000_000_u128, &onboarding_rebate)
        .unwrap(),
      416_666_666_666_666_u128.into()
    );

    let onboarding_rebate = OnboardingRebates {
      available_amount: 1_u128,
      initial_amount: 48_000_000_000_000_000_000_u128,
    };

    assert_noop!(
      Sunrise::get_next_onboarding_rebates(10_000_000_000_000_000_u128, &onboarding_rebate),
      Error::<Test>::NoRebatesAvailable
    );
  });
}

#[test]
pub fn test_try_refund_gas_for_deposit() {
  new_test_ext().execute_with(|| {
    let onboarding_rebate_before = Sunrise::onboarding();

    assert_ok!(Adapter::mint_into(
      CurrencyId::Tdfy,
      &Sunrise::account_id(),
      200_000_000_000_000_000,
    ));

    // 100k TDFY = 1 BTC
    // 100_000_000_000_000_000
    let oracle_value = 100_000_000_000_000_000_u128;
    assert_ok!(Sunrise::register_exchange_rate(vec![(2, oracle_value)]));

    assert_eq!(
      // 2 bitcoin @ oracle_value
      Sunrise::try_refund_gas_for_deposit(&ALICE_ACCOUNT_ID, CurrencyId::Wrapped(2), 200_000_000),
      Ok(Some(200_000_000_000_000_000))
    );

    let onboarding_rebate_after = Sunrise::onboarding();
    assert_eq!(
      onboarding_rebate_before
        .available_amount
        .saturating_sub(onboarding_rebate_after.available_amount),
      200_000_000_000_000_000
    );
  });
}

#[test]
pub fn test_try_refund_gas_for_deposit_should_fails() {
  new_test_ext().execute_with(|| {
    // 100k TDFY = 1 BTC
    // 100_000_000_000_000_000
    let oracle_value = 100_000_000_000_000_000_u128;
    assert_ok!(Sunrise::register_exchange_rate(vec![(2, oracle_value)]));

    assert_noop!(
      Sunrise::try_refund_gas_for_deposit(&ALICE_ACCOUNT_ID, CurrencyId::Wrapped(2), 200_000_000),
      BalancesError::<Test>::InsufficientBalance
    );
  });
}

#[test]
pub fn test_try_allocate_rewards_for_swap() {
  new_test_ext().execute_with(|| {
    let initial_first_pool_balance = Sunrise::sunrise_pools()
      .iter()
      .find(|pool| pool.id == 2)
      .unwrap()
      .balance;

    let initial_second_pool_balance = Sunrise::sunrise_pools()
      .iter()
      .find(|pool| pool.id == 1)
      .unwrap()
      .balance;

    // 0.002 BTC / TDFY
    let oracle_value = 500_000_000_000_000_u128;
    assert_ok!(Sunrise::register_exchange_rate(vec![(2, oracle_value)]));
    assert_eq!(Sunrise::pools_left_over(), 0);

    // Note: we're paying 1 BTC in fee for a 1_000 BTC swap (0.1%)
    let fee = Fee {
      amount: 100_000_000_000,
      fee: 100_000_000,
      fee_tdfy: Sunrise::try_get_tdfy_value(CurrencyId::Wrapped(2), 100_000_000).unwrap(),
    };

    // 1 BTC = 500 TDFY
    // 1 BTC @ 200% (pool id 2) = 2 BTC = 1_000 TDFY
    let expected_reward_first_pool = 1_000_000_000_000_000_u128;

    // 1 BTC = 500 TDFY
    // 1 BTC @ 125% (pool id 1) = 1.25 BTC = 625 TDFY
    let expected_reward_second_pool = 625_000_000_000_000;

    // 1 BTC = 500 TDFY
    // 1 BTC @ 50% (leftover) = 0.5 BTC = 250 TDFY
    let expected_reward_leftover = 250_000_000_000_000;

    // should have selected second pool
    assert_eq!(
      Sunrise::try_allocate_rewards_for_swap(&ALICE_ACCOUNT_ID, 1, &fee, CurrencyId::Wrapped(2))
        .unwrap()
        .unwrap(),
      expected_reward_first_pool
    );

    // confirm the pool has been updated
    assert_eq!(
      Sunrise::sunrise_pools()
        .iter()
        .find(|pool| pool.id == 2)
        .unwrap()
        .transactions_remaining,
      0
    );

    // reached transaction limit, the left over should be available
    assert_eq!(
      Sunrise::pools_left_over(),
      initial_first_pool_balance.saturating_sub(expected_reward_first_pool)
    );

    // should have selected second pool
    assert_eq!(
      Sunrise::try_allocate_rewards_for_swap(&ALICE_ACCOUNT_ID, 1, &fee, CurrencyId::Wrapped(2))
        .unwrap()
        .unwrap(),
      expected_reward_second_pool
    );

    // confirm the pool has been updated
    assert_eq!(
      Sunrise::sunrise_pools()
        .iter()
        .find(|pool| pool.id == 1)
        .unwrap()
        .transactions_remaining,
      0
    );

    // reached transaction limit, the left over should be available
    let expected_leftover_balance = initial_first_pool_balance
      .saturating_sub(expected_reward_first_pool)
      .saturating_add(initial_second_pool_balance.saturating_sub(expected_reward_second_pool));

    assert_eq!(Sunrise::pools_left_over(), expected_leftover_balance);

    // should have hitted the left-over
    assert_eq!(
      Sunrise::try_allocate_rewards_for_swap(&ALICE_ACCOUNT_ID, 1, &fee, CurrencyId::Wrapped(2))
        .unwrap()
        .unwrap(),
      expected_reward_leftover
    );

    assert_eq!(
      Sunrise::pools_left_over(),
      expected_leftover_balance.saturating_sub(expected_reward_leftover)
    );
  });
}
