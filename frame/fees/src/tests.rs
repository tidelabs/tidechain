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
  mock::{new_test_ext, AccountId, Adapter, ExistentialDeposit, Fees, Origin, Test},
  ActiveEra, AssetExchangeRate, Error, SunrisePools, SunriseRewards,
};
use frame_support::{
  assert_noop, assert_ok,
  traits::fungibles::{Inspect, Mutate},
};
use pallet_balances::Error as BalancesError;
use sp_runtime::{
  traits::{BadOrigin, CheckedDiv},
  FixedPointNumber, FixedU128,
};
use tidefi_primitives::{
  assets::Asset, pallet::FeesExt, ActiveEraInfo, CurrencyId, EraIndex, SwapType,
};

type Balance = u128;
type BlockNumber = u64;

// Asset Units
const ONE_TDFY: Balance = 1_000_000_000_000;

// Test Accounts
const CHARLIE_ACCOUNT_ID: AccountId = AccountId(3u64);

struct Context {
  rewards_claimer: Origin,
  era_index: EraIndex,
}

impl Default for Context {
  fn default() -> Self {
    Self {
      rewards_claimer: Origin::signed(CHARLIE_ACCOUNT_ID),
      era_index: 1,
    }
  }
}

impl Context {
  fn mint_tdfy(self, account: AccountId, amount: Balance) -> Self {
    let balance_before_mint = Adapter::balance(CurrencyId::Tdfy, &account);
    Self::mint_asset_for_accounts(vec![account], CurrencyId::Tdfy, amount);
    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &account),
      balance_before_mint + amount
    );
    self
  }

  fn mint_asset_for_accounts(accounts: Vec<AccountId>, asset: CurrencyId, amount: u128) {
    for account in accounts {
      assert_ok!(Adapter::mint_into(asset, &account, amount));
    }
  }

  fn set_active_era(self, era_index: u32, start_block: BlockNumber) -> Self {
    ActiveEra::<Test>::put(ActiveEraInfo::<BlockNumber> {
      index: era_index,
      start_block: Some(start_block),
      start_session_index: None,
      last_session_block: None,
      start: None,
    });
    self
  }

  fn set_sunrise_rewards(
    self,
    account: AccountId,
    era_index: u32,
    rewards_amount: Balance,
  ) -> Self {
    SunriseRewards::<Test>::insert(account, era_index, rewards_amount);
    self
  }
}

mod claim_sunrise_rewards {
  use super::*;

  #[test]
  fn succeeds() {
    new_test_ext().execute_with(|| {
      let context = Context::default()
        .mint_tdfy(Fees::account_id(), 1_000 * ONE_TDFY)
        .mint_tdfy(CHARLIE_ACCOUNT_ID, 1_000 * ONE_TDFY)
        .set_active_era(3, 1)
        .set_sunrise_rewards(CHARLIE_ACCOUNT_ID, 1, ONE_TDFY);

      assert_ok!(Fees::claim_sunrise_rewards(
        context.rewards_claimer,
        context.era_index,
      ));
    });
  }

  mod fails_when {
    use super::*;

    #[test]
    fn not_signed() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(Fees::account_id(), 1_000 * ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, 1_000 * ONE_TDFY)
          .set_active_era(3, 1)
          .set_sunrise_rewards(CHARLIE_ACCOUNT_ID, 1, ONE_TDFY);

        assert_noop!(
          Fees::claim_sunrise_rewards(Origin::none(), context.era_index),
          BadOrigin
        );
      });
    }

    #[test]
    fn no_active_era() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(Fees::account_id(), 1_000 * ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, 1_000 * ONE_TDFY)
          .set_active_era(3, 1)
          .set_sunrise_rewards(CHARLIE_ACCOUNT_ID, 1, ONE_TDFY);

        ActiveEra::<Test>::kill();

        assert_noop!(
          Fees::claim_sunrise_rewards(context.rewards_claimer, context.era_index),
          Error::<Test>::NoActiveEra
        );
      });
    }

    #[test]
    fn no_active_era_start_block() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(Fees::account_id(), 1_000 * ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, 1_000 * ONE_TDFY)
          .set_sunrise_rewards(CHARLIE_ACCOUNT_ID, 1, ONE_TDFY);

        assert_noop!(
          Fees::claim_sunrise_rewards(context.rewards_claimer, context.era_index),
          Error::<Test>::NoActiveEraStartBlock
        );
      });
    }

    #[test]
    fn claim_current_era() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(Fees::account_id(), 1_000 * ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, 1_000 * ONE_TDFY)
          .set_active_era(3, 1)
          .set_sunrise_rewards(CHARLIE_ACCOUNT_ID, 1, ONE_TDFY);

        let current_era = Fees::active_era().unwrap().index;

        assert_noop!(
          Fees::claim_sunrise_rewards(context.rewards_claimer.clone(), current_era),
          Error::<Test>::InvalidEra
        );
      });
    }

    #[test]
    fn claim_future_era() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(Fees::account_id(), 1_000 * ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, 1_000 * ONE_TDFY)
          .set_active_era(3, 1)
          .set_sunrise_rewards(CHARLIE_ACCOUNT_ID, 1, ONE_TDFY);

        let future_era = Fees::active_era().unwrap().index + 1;

        assert_noop!(
          Fees::claim_sunrise_rewards(context.rewards_claimer, future_era),
          Error::<Test>::InvalidEra
        );
      });
    }

    #[test]
    fn claim_previous_era_without_blocks_sunrise_claims_cooldown_completed() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(Fees::account_id(), 1_000 * ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, 1_000 * ONE_TDFY)
          .set_active_era(3, 1)
          .set_sunrise_rewards(CHARLIE_ACCOUNT_ID, 1, ONE_TDFY);

        let previous_era = Fees::active_era().unwrap().index - 1;

        assert_noop!(
          Fees::claim_sunrise_rewards(context.rewards_claimer.clone(), previous_era),
          Error::<Test>::EraNotReady
        );
      });
    }

    #[test]
    fn reward_is_zero() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(Fees::account_id(), 1_000 * ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, 1_000 * ONE_TDFY)
          .set_active_era(3, 1)
          .set_sunrise_rewards(CHARLIE_ACCOUNT_ID, 1, 0);

        assert_noop!(
          Fees::claim_sunrise_rewards(context.rewards_claimer, context.era_index),
          Error::<Test>::NoRewardsAvailable
        );
      });
    }

    #[test]
    fn no_rewards_available() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(Fees::account_id(), 1_000 * ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, 1_000 * ONE_TDFY)
          .set_active_era(3, 1);

        assert_noop!(
          Fees::claim_sunrise_rewards(context.rewards_claimer, context.era_index),
          Error::<Test>::NoRewardsAvailable
        );
      });
    }

    #[test]
    fn fees_pallet_account_has_insufficient_balance_to_pay_rewards() {
      new_test_ext().execute_with(|| {
        Context::default()
          .mint_tdfy(CHARLIE_ACCOUNT_ID, 1_000 * ONE_TDFY)
          .set_active_era(3, 1);

        let fees_pallet_account_balance = Adapter::balance(CurrencyId::Tdfy, &Fees::account_id());
        let context = Context::default().set_sunrise_rewards(
          CHARLIE_ACCOUNT_ID,
          1,
          fees_pallet_account_balance + 1,
        );

        assert_noop!(
          Fees::claim_sunrise_rewards(context.rewards_claimer.clone(), context.era_index),
          BalancesError::<Test>::InsufficientBalance
        );
      });
    }

    #[test]
    fn claimer_account_cannot_keep_alive() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(Fees::account_id(), 1_000 * ONE_TDFY)
          .set_active_era(3, 1)
          .set_sunrise_rewards(CHARLIE_ACCOUNT_ID, 1, ExistentialDeposit::get() - 1);

        assert_noop!(
          Fees::claim_sunrise_rewards(context.rewards_claimer.clone(), context.era_index),
          BalancesError::<Test>::ExistentialDeposit
        );
      });
    }

    #[test]
    fn fees_pallet_account_cannot_keep_alive() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(CHARLIE_ACCOUNT_ID, 1_000 * ONE_TDFY)
          .set_active_era(3, 1)
          .set_sunrise_rewards(CHARLIE_ACCOUNT_ID, 1, ONE_TDFY);

        assert_noop!(
          Fees::claim_sunrise_rewards(context.rewards_claimer, context.era_index,),
          BalancesError::<Test>::KeepAlive
        );
      });
    }
  }
}

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
    AssetExchangeRate::<Test>::insert(4, FixedU128::saturating_from_rational(1, 1));

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
    assert_eq!(
      SunrisePools::<Test>::get()
        .first()
        .unwrap()
        .transactions_remaining,
      1
    );
    assert_eq!(
      SunrisePools::<Test>::get().first().unwrap().balance,
      67_200_000_000_000_000_000_u128
    );

    // 1 USDT = 1 TDFY
    AssetExchangeRate::<Test>::insert(4, FixedU128::saturating_from_rational(1, 1));

    let current_era = Fees::active_era().unwrap().index;
    Fees::start_era();
    let new_current_era = Fees::active_era().unwrap().index;
    assert_eq!(current_era + 1, new_current_era);

    // 100 tifi @ 2% should cost 2 TIFIs
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
      SunrisePools::<Test>::get()
        .first()
        .unwrap()
        .transactions_remaining,
      0
    );
    assert_eq!(
      SunrisePools::<Test>::get().first().unwrap().balance,
      67_199_997_500_000_000_000
    );
  });
}

#[test]
pub fn test_calc_reward() {
  new_test_ext().execute_with(|| {
    // 0.7 USDT = 1 TDFY
    AssetExchangeRate::<Test>::insert(4, FixedU128::saturating_from_rational(700_000, 1_000_000));

    let fee =
      Fees::calculate_swap_fees(CurrencyId::Wrapped(4), 100_000_000, SwapType::Limit, false);
    assert_eq!(
      Fees::calculate_rebates_on_fees_paid(
        // 125%
        FixedU128::saturating_from_rational(125, 100),
        // 2$ USDT in fee
        // Should have total 2.5$ USDT in reward
        // 2.5 / 0.7 = 3.57142857143 TDFY final
        &fee,
      )
      .unwrap(),
      1_750_000_000_000
    );
  });
}

#[test]
pub fn test_calc_reward_small_numbers() {
  new_test_ext().execute_with(|| {
    // 2 TDFY / USDT
    let oracle_value = 2_000_000_000_000_u128;
    let how_many_tdfy_for_one_usdt =
      FixedU128::saturating_from_rational(oracle_value, Asset::Tdfy.saturating_mul(1));
    assert_eq!(how_many_tdfy_for_one_usdt.to_float(), 2.0);
    AssetExchangeRate::<Test>::insert(4, how_many_tdfy_for_one_usdt);

    let fee = Fees::calculate_swap_fees(CurrencyId::Wrapped(4), 1_000_000, SwapType::Limit, false);

    // We should receive 0.15625 TDFY in reward
    assert_eq!(
      Fees::calculate_rebates_on_fees_paid(
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
    let how_many_tdfy_for_one_bitcoin =
      FixedU128::saturating_from_rational(oracle_value, Asset::Tdfy.saturating_mul(1));
    assert_eq!(how_many_tdfy_for_one_bitcoin.to_float(), 99999.99999999999);
    AssetExchangeRate::<Test>::insert(2, how_many_tdfy_for_one_bitcoin);

    let fee =
      Fees::calculate_swap_fees(CurrencyId::Wrapped(2), 100_000_000, SwapType::Limit, false);

    // We should receive 2500 TDFY in reward
    assert_eq!(
      Fees::calculate_rebates_on_fees_paid(
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
      Fees::calculate_rebates_on_fees_paid(
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
    let how_many_tdfy_for_one_bitcoin =
      FixedU128::saturating_from_rational(oracle_value, Asset::Tdfy.saturating_mul(1));
    assert_eq!(how_many_tdfy_for_one_bitcoin.to_float(), 550.9641873278);

    AssetExchangeRate::<Test>::insert(2, how_many_tdfy_for_one_bitcoin);

    // 1 BTC should equal value set
    assert_eq!(
      Fees::try_get_tdfy_value(CurrencyId::Wrapped(2), Asset::Bitcoin.saturating_mul(1)).unwrap(),
      oracle_value.into()
    );

    let fee = Fees::calculate_swap_fees(
      CurrencyId::Wrapped(2),
      // 200_000 BTC
      Asset::Bitcoin.saturating_mul(200_000),
      SwapType::Limit,
      false,
    );

    let reward = Fees::calculate_rebates_on_fees_paid(
      // 125%
      FixedU128::saturating_from_rational(125, 100),
      &fee,
    )
    .unwrap();

    assert_eq!(reward, 12_500_000_000_000_000);
  });
}

#[test]
pub fn test_wrapped_asset_values() {
  new_test_ext().execute_with(|| {
    // 100k TDFY = 1 BTC
    let oracle_value = 100_000_000_000_000_000_u128;
    let btc_price =
      FixedU128::saturating_from_rational(oracle_value, Asset::Tdfy.saturating_mul(1));
    AssetExchangeRate::<Test>::insert(2, btc_price);

    assert_eq!(
      Fees::try_get_tdfy_value(CurrencyId::Wrapped(2), 100_000_000).unwrap(),
      oracle_value.into()
    );

    assert_eq!(
      Fees::try_get_tdfy_value(CurrencyId::Wrapped(2), 10_000_000).unwrap(),
      10_000_000_000_000_000_u128.into()
    );

    assert_eq!(
      Fees::try_get_tdfy_value(CurrencyId::Wrapped(2), 625_000).unwrap(),
      625_000_000_000_000_u128.into()
    );
  });
}

#[test]
pub fn test_saturation_and_conversion() {
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
    Fees::convert_fixed_balance_to_tdfy_balance(total_tdfy_for_one_eth).unwrap(),
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
    Fees::convert_fixed_balance_to_tdfy_balance(total_tdfy_for_one_usdt).unwrap(),
    29_717_682_020
  );
}

#[test]
pub fn test_select_first_eligible_sunrise_pool() {
  new_test_ext().execute_with(|| {
    assert_eq!(SunrisePools::<Test>::get().len(), 2);

    // 0.001815 BTC for 1 TDFY
    let oracle_value = 550_964_187_327_800_u128;
    let how_many_tdfy_for_one_bitcoin =
      FixedU128::saturating_from_rational(oracle_value, Asset::Tdfy.saturating_mul(1));
    assert_eq!(how_many_tdfy_for_one_bitcoin.to_float(), 550.9641873278);

    AssetExchangeRate::<Test>::insert(2, how_many_tdfy_for_one_bitcoin);

    // 1 BTC should equal value set
    assert_eq!(
      Fees::try_get_tdfy_value(CurrencyId::Wrapped(2), Asset::Bitcoin.saturating_mul(1)).unwrap(),
      oracle_value.into()
    );

    // should have selected second pool
    assert_eq!(
      Fees::try_select_first_eligible_sunrise_pool(
        &Fees::calculate_swap_fees(
          CurrencyId::Wrapped(2),
          // 10 BTC
          Asset::Bitcoin.saturating_mul(10),
          SwapType::Limit,
          false,
        ),
        CurrencyId::Wrapped(2)
      )
      .unwrap()
      .unwrap()
      .id,
      2
    );

    // should have selected first pool
    assert_eq!(
      Fees::try_select_first_eligible_sunrise_pool(
        &Fees::calculate_swap_fees(
          CurrencyId::Wrapped(2),
          // 1 BTC
          Asset::Bitcoin.saturating_mul(1),
          SwapType::Limit,
          false,
        ),
        CurrencyId::Wrapped(2)
      )
      .unwrap()
      .unwrap()
      .id,
      1
    );
  })
}
