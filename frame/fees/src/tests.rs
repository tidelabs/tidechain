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

use crate::mock::{new_test_ext, AccountId, Adapter, Fees, Sunrise, Test, TidefiStaking};
use frame_support::{
  assert_ok,
  traits::fungibles::{Inspect, Mutate},
};
use sp_runtime::{FixedPointNumber, FixedU128};
use tidefi_primitives::{
  assets::Asset,
  pallet::{FeesExt, SunriseExt},
  CurrencyId, Stake, StakeStatus, SwapType,
};

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    assert!(Fees::current_era().is_some());
  });
}

#[test]
pub fn calculate_trading_fees() {
  new_test_ext().execute_with(|| {
    // 1 USDT = 1 TDFY
    assert_ok!(Sunrise::register_exchange_rate(vec![(
      4,
      1_000_000_000_000
    )]));

    // 100 TDFY @ 2% should cost 2 TDFY
    let calculated_fee = Fees::calculate_swap_fees(
      CurrencyId::Tdfy,
      100_000_000_000_000,
      SwapType::Limit,
      false,
    );
    assert_eq!(calculated_fee.amount, 100_000_000_000_000);
    assert_eq!(calculated_fee.fee, 2_000_000_000_000);

    let calculated_fee =
      Fees::calculate_swap_fees(CurrencyId::Tdfy, 100_000_000_000_000, SwapType::Limit, true);
    assert_eq!(calculated_fee.amount, 100_000_000_000_000);
    assert_eq!(calculated_fee.fee, 1_000_000_000_000);
  });
}

#[test]
pub fn register_swap_fees() {
  new_test_ext().execute_with(|| {
    assert_eq!(
      Sunrise::sunrise_pools()
        .first()
        .unwrap()
        .transactions_remaining,
      1
    );
    assert_eq!(
      Sunrise::sunrise_pools().first().unwrap().balance,
      67_200_000_000_000_000_000_u128
    );

    // 1 USDT = 1 TDFY
    assert_ok!(Sunrise::register_exchange_rate(vec![(
      4,
      1_000_000_000_000
    )]));

    let current_era = Fees::current_era().unwrap().index;
    Fees::start_era();
    let new_current_era = Fees::current_era().unwrap().index;
    assert_eq!(current_era + 1, new_current_era);

    // 100 TDFY @ 2% should cost 2 TDFY
    let calculated_fee = Fees::register_swap_fees(
      3u64.into(),
      CurrencyId::Wrapped(4),
      100_000_000,
      SwapType::Limit,
      false,
    )
    .unwrap();
    assert_eq!(calculated_fee.amount, 100_000_000);
    assert_eq!(calculated_fee.fee, 2_000_000);
    // make sure everything was registered
    let registered_fee = Fees::account_fees(new_current_era, AccountId(3u64));
    assert_eq!(registered_fee.first().unwrap().1.fee, 2_000_000);

    assert_eq!(registered_fee.first().unwrap().1.amount, 100_000_000);

    // make sure it increment the value
    assert_ok!(Fees::register_swap_fees(
      3u64.into(),
      CurrencyId::Tdfy,
      100_000_000,
      SwapType::Limit,
      false,
    ));

    let registered_fee = Fees::account_fees(new_current_era, AccountId(3u64));
    assert_eq!(registered_fee.first().unwrap().1.amount, 100_000_000);
    assert_eq!(registered_fee.first().unwrap().1.fee, 2_000_000);
    assert_eq!(
      registered_fee.first().unwrap().1.fee_tdfy,
      2_000_000_000_000
    );
    assert_eq!(
      Sunrise::sunrise_pools()
        .first()
        .unwrap()
        .transactions_remaining,
      0
    );
    assert_eq!(Sunrise::sunrise_pools().first().unwrap().balance, 0);
    assert_eq!(Sunrise::pools_left_over(), 67_199_997_499_999_000_000);
  });
}

#[test]
pub fn test_calc_reward() {
  new_test_ext().execute_with(|| {
    // 1 TDFY = 0.7 USDT
    let oracle_value = 1_428_571_428_600_u128;
    assert_ok!(Sunrise::register_exchange_rate(vec![(4, oracle_value)]));

    let fee =
      Fees::calculate_swap_fees(CurrencyId::Wrapped(4), 100_000_000, SwapType::Limit, false);

    assert_eq!(
      Sunrise::calculate_rebates_on_fees_paid(
        // 125%
        FixedU128::saturating_from_rational(125, 100),
        &fee,
      )
      .unwrap(),
      3_571_428_571_500
    );
  });
}

#[test]
pub fn test_calc_reward_small_numbers() {
  new_test_ext().execute_with(|| {
    // 2 TDFY / USDT
    let oracle_value = 2_000_000_000_000_u128;
    assert_ok!(Sunrise::register_exchange_rate(vec![(4, oracle_value)]));

    let fee = Fees::calculate_swap_fees(CurrencyId::Wrapped(4), 1_000_000, SwapType::Limit, false);

    // We should receive 0.15625 TDFY in reward
    assert_eq!(
      Sunrise::calculate_rebates_on_fees_paid(
        // 125%
        FixedU128::saturating_from_rational(125, 100),
        // 0.2 in fee
        // Should have total 0.25$ USDT in reward
        // 0.25 / 0.5 = 0.5 TDFY final
        &fee,
      )
      .unwrap(),
      50_000_000_000
    );
  });
}

#[test]
pub fn test_calc_reward_other_assets() {
  new_test_ext().execute_with(|| {
    // 100k TDFY / 1 BTC
    let oracle_value = 100_000_000_000_000_000_u128;
    assert_ok!(Sunrise::register_exchange_rate(vec![(2, oracle_value)]));

    let fee =
      Fees::calculate_swap_fees(CurrencyId::Wrapped(2), 100_000_000, SwapType::Limit, false);

    // We should receive 2500 TDFY in reward
    assert_eq!(
      Sunrise::calculate_rebates_on_fees_paid(
        // 125%
        FixedU128::saturating_from_rational(125_u128, 100_u128),
        // 0.02 BTC in fee
        // Should have total 0.025BTC in reward
        // 0.025 * 100_000 = 2500 TDFY
        &fee,
      )
      .unwrap(),
      2_500_000_000_000_000
    );

    // 1_000 BTC transaction
    // worth 10_000_000 USDT
    let fee = Fees::calculate_swap_fees(
      CurrencyId::Wrapped(2),
      100_000_000_000,
      SwapType::Limit,
      false,
    );

    // We should receive 125_000 TDFY in reward
    assert_eq!(
      Sunrise::calculate_rebates_on_fees_paid(
        // 225%
        FixedU128::saturating_from_rational(225_u128, 100_u128),
        // 1.25 BTC in reward
        &fee,
      )
      .unwrap(),
      22_500_000_000_000_000
    );
  });
}

#[test]
pub fn test_maximum_fee_values() {
  new_test_ext().execute_with(|| {
    // 0.001815 BTC for 1 TDFY
    let oracle_value = 550_964_187_327_800_u128;
    assert_ok!(Sunrise::register_exchange_rate(vec![(2, oracle_value)]));

    // 1 BTC should equal value set
    assert_eq!(
      Sunrise::try_get_tdfy_value(CurrencyId::Wrapped(2), Asset::Bitcoin.saturating_mul(1))
        .unwrap(),
      oracle_value
    );

    let fee = Fees::calculate_swap_fees(
      CurrencyId::Wrapped(2),
      // 200_000 BTC
      Asset::Bitcoin.saturating_mul(200_000),
      SwapType::Limit,
      false,
    );

    let reward = Sunrise::calculate_rebates_on_fees_paid(
      // 125%
      FixedU128::saturating_from_rational(125, 100),
      &fee,
    )
    .unwrap();

    assert_eq!(reward, 12_500_000_000_000_000);
  });
}

#[test]
fn test_migration_v2() {
  new_test_ext().execute_with(|| {
    let pallet_account_id = Fees::account_id();
    let staking_pool_account_id = TidefiStaking::account_id();
    let initial_tdfy_to_fees = 98_000_000_000_000;
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tdfy,
      &pallet_account_id,
      initial_tdfy_to_fees,
    ));

    // create 100 stake
    let mut total_fees = 0;
    let mut total_staked = 0;
    (0..100).for_each(|i| {
      let account = frame_benchmarking::account::<AccountId>("stake", i, 1);
      assert_ok!(Adapter::mint_into(
        CurrencyId::Wrapped(2),
        &account,
        500_000_000_u128.saturating_mul(2_u128),
      ));

      let fees = 10_u128.saturating_add(i.into());
      let initial_balance = 500_000_000_u128;
      let principal = initial_balance.saturating_add(fees);

      // mint principal (stake) to staking pallet
      assert_ok!(Adapter::mint_into(
        CurrencyId::Wrapped(2),
        &staking_pool_account_id,
        initial_balance,
      ));

      // mint fees to fees pallet
      assert_ok!(Adapter::mint_into(
        CurrencyId::Wrapped(2),
        &pallet_account_id,
        fees,
      ));

      // append the stake into the storage
      assert_ok!(pallet_tidefi_stake::AccountStakes::<Test>::try_append(
        account,
        Stake {
          currency_id: CurrencyId::Wrapped(2),
          unique_id: Default::default(),
          last_session_index_compound: 1,
          initial_block: 1,
          initial_balance,
          principal,
          duration: 0,
          status: StakeStatus::Staked,
        }
      ));

      total_fees += fees;
      total_staked += initial_balance;
    });

    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Wrapped(2), &pallet_account_id, false),
      total_fees
    );
    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Wrapped(2), &staking_pool_account_id, false),
      total_staked
    );
    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Tdfy, &pallet_account_id, true),
      initial_tdfy_to_fees
    );

    // run migration
    crate::migrations::v2::migrate::<Test>();
    crate::migrations::v2::post_migration::<Test>();

    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Wrapped(2), &pallet_account_id, false),
      0
    );
    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Wrapped(2), &staking_pool_account_id, false),
      total_staked.saturating_add(total_fees)
    );
    // make sure we still have our keep alive
    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Tdfy, &pallet_account_id, false),
      1_000_000_000_000
    );
    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Tdfy, &pallet_account_id, true),
      0
    );
  });
}
