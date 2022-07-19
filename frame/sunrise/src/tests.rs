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

use crate::mock::{new_test_ext, Sunrise};
use frame_support::assert_ok;
use sp_runtime::{traits::CheckedDiv, FixedPointNumber, FixedU128};
use tidefi_primitives::{assets::Asset, pallet::SunriseExt, CurrencyId, Fee};

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
