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
  mock::{new_test_ext, AccountId, Fees, Test},
  OrderBookPrice,
};
use frame_support::assert_ok;
use sp_runtime::{FixedPointNumber, FixedU128};
use tidefi_primitives::{pallet::FeesExt, CurrencyId, SwapType};

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    assert!(Fees::active_era().is_some());
  });
}

#[test]
pub fn calculate_trading_fees() {
  new_test_ext().execute_with(|| {
    // 1 USDT = 1 TDFY
    OrderBookPrice::<Test>::insert(
      CurrencyId::Tdfy,
      CurrencyId::Wrapped(4),
      FixedU128::saturating_from_rational(1, 1),
    );

    // 100 tifi @ 2% should cost 2 TIFIs
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
    // 1 USDT = 1 TDFY
    OrderBookPrice::<Test>::insert(
      CurrencyId::Tdfy,
      CurrencyId::Wrapped(4),
      FixedU128::saturating_from_rational(1, 1),
    );

    let current_era = Fees::active_era().unwrap().index;
    Fees::start_era();
    let new_current_era = Fees::active_era().unwrap().index;
    assert_eq!(current_era + 1, new_current_era);

    // 100 tifi @ 2% should cost 2 TIFIs
    let calculated_fee = Fees::register_swap_fees(
      3u64.into(),
      CurrencyId::Tdfy,
      100_000_000_000_000,
      SwapType::Limit,
      false,
    )
    .unwrap();
    assert_eq!(calculated_fee.amount, 100_000_000_000_000);
    assert_eq!(calculated_fee.fee, 2_000_000_000_000);

    // make sure everything was registered
    let registered_fee = Fees::account_fees(new_current_era, AccountId(3u64));
    assert_eq!(registered_fee.first().unwrap().1.fee, 2_000_000_000_000);

    assert_eq!(
      registered_fee.first().unwrap().1.amount,
      100_000_000_000_000
    );

    // make sure it increment the value
    assert_ok!(Fees::register_swap_fees(
      3u64.into(),
      CurrencyId::Tdfy,
      100_000_000_000_000,
      SwapType::Limit,
      false
    ));
    let registered_fee = Fees::account_fees(new_current_era, AccountId(3u64));
    assert_eq!(
      registered_fee.first().unwrap().1.amount,
      200_000_000_000_000
    );
    assert_eq!(registered_fee.first().unwrap().1.fee, 4_000_000_000_000);
  });
}

#[test]
pub fn test_calc_reward() {
  new_test_ext().execute_with(|| {
    // 0.7 USDT = 1 TDFY
    OrderBookPrice::<Test>::insert(
      CurrencyId::Wrapped(4),
      CurrencyId::Tdfy,
      FixedU128::saturating_from_rational(700_000, 1_000_000),
    );

    let fee =
      Fees::calculate_swap_fees(CurrencyId::Wrapped(4), 100_000_000, SwapType::Limit, false);
    assert_eq!(
      Fees::calculate_tide_reward_for_pool(
        // 125%
        FixedU128::saturating_from_rational(125, 100),
        // 2$ USDT in fee
        // Should have total 2.5$ USDT in reward
        // 2.5 / 0.7 = 3.57142857143 TDFY final
        &fee,
        CurrencyId::Wrapped(4)
      )
      .unwrap(),
      3_571_428_571_428
    );
  });
}

#[test]
pub fn test_calc_reward_small_numbers() {
  new_test_ext().execute_with(|| {
    // 0.5 USDT = 1 TDFY
    OrderBookPrice::<Test>::insert(
      CurrencyId::Wrapped(4),
      CurrencyId::Tdfy,
      FixedU128::saturating_from_rational(500_000, 1_000_000),
    );

    let fee = Fees::calculate_swap_fees(CurrencyId::Wrapped(4), 1_000_000, SwapType::Limit, false);

    // We should receive 0.15625 TDFY in reward
    assert_eq!(
      Fees::calculate_tide_reward_for_pool(
        // 125%
        FixedU128::saturating_from_rational(125, 100),
        // 0.2 in fee
        // Should have total 0.25$ USDT in reward
        // 0.25 / 0.5 = 0.5 TDFY final
        &fee,
        CurrencyId::Wrapped(4)
      )
      .unwrap(),
      50_000_000_000
    );
  });
}

#[test]
pub fn test_calc_reward_other_assets() {
  new_test_ext().execute_with(|| {
    // 10_000 USDT = 1 BTC
    OrderBookPrice::<Test>::insert(
      CurrencyId::Wrapped(2),
      CurrencyId::Wrapped(4),
      FixedU128::saturating_from_rational(1, 10_000),
    );

    // 100_000 TDFY = 1 BTC
    OrderBookPrice::<Test>::insert(
      CurrencyId::Wrapped(2),
      CurrencyId::Tdfy,
      FixedU128::saturating_from_rational(1, 100_000),
    );

    let fee =
      Fees::calculate_swap_fees(CurrencyId::Wrapped(2), 100_000_000, SwapType::Limit, false);
    // We should receive 2500 TDFY in reward
    assert_eq!(
      Fees::calculate_tide_reward_for_pool(
        // 125%
        FixedU128::saturating_from_rational(125, 100),
        // 0.02 BTC in fee
        // Should have total 0.025BTC in reward
        // 0.025 * 100_000 = 2500 TDFY
        &fee,
        CurrencyId::Wrapped(2)
      )
      .unwrap(),
      2_500_000_000_000_000
    );

    // 1 BTC max at 10k USDT each
    let max_amount_in_btc =
      Fees::try_get_value_from_usdt(CurrencyId::Wrapped(2), 10_000_000_000_u128.into()).unwrap();

    assert_eq!(max_amount_in_btc, 100_000_000);

    // TDFY amount
    let max_amount_in_tide =
      Fees::try_get_tide_value(CurrencyId::Wrapped(2), max_amount_in_btc.into()).unwrap();

    // 100k TDFY maximum fees allocation
    assert_eq!(max_amount_in_tide, 100_000_000_000_000_000);

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
      Fees::calculate_tide_reward_for_pool(
        // 125% of 10k USDT = 12_500
        FixedU128::saturating_from_rational(125, 100),
        // 1.25 BTC in reward
        &fee,
        CurrencyId::Wrapped(2)
      )
      .unwrap(),
      125_000_000_000_000_000
    );
  });
}
