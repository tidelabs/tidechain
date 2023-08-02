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
  mock::{
    new_test_ext, AccountId, Adapter, Balances, FeeAmount, Fees, MarketMakerFeeAmount,
    MarketMakerLimitFeeAmount, Oracle, RuntimeEvent as MockEvent, RuntimeOrigin, Sunrise, System,
    Test,
  },
  pallet::*,
};
use frame_support::{
  assert_noop, assert_ok,
  traits::fungibles::{Inspect, InspectHold, Mutate, MutateHold},
  BoundedVec,
};
use pallet_assets::Account;
use sp_core::H256;
use sp_runtime::{
  traits::{BadOrigin, Zero},
  FixedPointNumber, FixedU128, Permill,
};
use std::str::FromStr;
use tidefi_primitives::{
  pallet::{FeesExt, OracleExt},
  Balance, CurrencyId, Hash, MarketPair, Swap, SwapConfirmation, SwapStatus, SwapType,
};

const CURRENT_BLOCK_NUMBER: BlockNumber = 0;

// TEMP Asset
const TEMP_ASSET_ID: u32 = 5;
const TEMP_CURRENCY_ID: CurrencyId = CurrencyId::Wrapped(TEMP_ASSET_ID);
const ONE_TEMP: u128 = 1_000_000;

// TEMP2 Asset
const TEMP2_ASSET_ID: u32 = 2;
const TEMP2_CURRENCY_ID: CurrencyId = CurrencyId::Wrapped(TEMP2_ASSET_ID);
const ONE_TEMP2: u128 = 100_000_000;

// ZEMP Asset
const ZEMP_ASSET_ID: u32 = 3;
const ZEMP_CURRENCY_ID: CurrencyId = CurrencyId::Wrapped(ZEMP_ASSET_ID);
const ONE_ZEMP: u128 = 1_000_000_000_000_000_000;

const ONE_TDFY: u128 = 1_000_000_000_000;

// Test Accounts
const ALICE_ACCOUNT_ID: AccountId = AccountId(1);
const BOB_ACCOUNT_ID: AccountId = AccountId(2);
const CHARLIE_ACCOUNT_ID: AccountId = AccountId(3);
const DAVE_ACCOUNT_ID: AccountId = AccountId(4);

// Extrinsic Hashes
const EXTRINSIC_HASH_0: [u8; 32] = [0; 32];
const EXTRINSIC_HASH_1: [u8; 32] = [1; 32];
const EXTRINSIC_HASH_2: [u8; 32] = [2; 32];

const INITIAL_20_TDFYS: Balance = 20 * ONE_TDFY;
const INITIAL_10000_TEMPS: Balance = 10_000 * ONE_TEMP;
const BOB_SELLS_10_TDFYS: Balance = 10 * ONE_TDFY;
const BOB_BUYS_200_TEMPS: Balance = 200 * ONE_TEMP;
const CHARLIE_SELLS_4000_TEMPS: Balance = 4_000 * ONE_TEMP;
const CHARLIE_BUYS_200_TDFYS: Balance = 200 * ONE_TDFY;
const DAVE_PARTIAL_FILLING_SELLS_60_TEMPS: Balance = 60 * ONE_TEMP;
const DAVE_PARTIAL_FILLING_BUYS_3_TDFYS: Balance = 3 * ONE_TDFY;

// Swap Fee Rates
const REQUESTER_SWAP_FEE_RATE: Permill = FeeAmount::get();
const MARKET_MAKER_SWAP_FEE_RATE: Permill = MarketMakerFeeAmount::get();
const MARKET_MAKER_SWAP_LIMIT_FEE_RATE: Permill = MarketMakerLimitFeeAmount::get();

// Slippage Rates
const SLIPPAGE_0_PERCENT: Permill = Permill::from_percent(0);
const SLIPPAGE_2_PERCENTS: Permill = Permill::from_percent(2);
const SLIPPAGE_4_PERCENTS: Permill = Permill::from_percent(4);
const SLIPPAGE_5_PERCENTS: Permill = Permill::from_percent(5);

type BlockNumber = u64;

#[derive(Clone)]
struct Context {
  alice: RuntimeOrigin,
  bob: RuntimeOrigin,
  fees_account_id: AccountId,
}

impl Default for Context {
  fn default() -> Self {
    let fees_account_id = Fees::account_id();
    assert_eq!(fees_account_id, 8246216774960574317_u64.into());

    Self {
      alice: RuntimeOrigin::signed(ALICE_ACCOUNT_ID),
      bob: RuntimeOrigin::signed(BOB_ACCOUNT_ID),
      fees_account_id,
    }
  }
}

impl Context {
  fn set_oracle_status(self, status: bool) -> Self {
    assert_ok!(Oracle::set_status(self.alice.clone(), status));
    match status {
      true => assert!(Oracle::status()),
      false => assert!(!Oracle::status()),
    }
    self
  }

  fn set_market_makers(self, account_ids: Vec<AccountId>) -> Self {
    account_ids
      .iter()
      .for_each(|account_id| MarketMakers::<Test>::insert(account_id, true));
    self
  }

  fn mint_tdfy(self, account: AccountId, amount: u128) -> Self {
    let initial_balance_tdfy = Adapter::balance(CurrencyId::Tdfy, &account);
    Self::mint_asset_for_accounts(vec![account], CurrencyId::Tdfy, amount);
    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &account),
      initial_balance_tdfy + amount
    );
    self
  }

  fn mint_temp(self, account: AccountId, amount: u128) -> Self {
    Self::mint_asset_for_accounts(vec![account], TEMP_CURRENCY_ID, amount);
    assert_eq!(Adapter::balance(TEMP_CURRENCY_ID, &account), amount);
    self
  }

  fn mint_temp2(self, account: AccountId, amount: u128) -> Self {
    Self::mint_asset_for_accounts(vec![account], TEMP2_CURRENCY_ID, amount);
    assert_eq!(Adapter::balance(TEMP2_CURRENCY_ID, &account), amount);
    self
  }

  fn mint_zemp(self, account: AccountId, amount: u128) -> Self {
    Self::mint_asset_for_accounts(vec![account], ZEMP_CURRENCY_ID, amount);
    assert_eq!(Adapter::balance(ZEMP_CURRENCY_ID, &account), amount);
    self
  }

  fn mint_asset_for_accounts(accounts: Vec<AccountId>, asset: CurrencyId, amount: u128) {
    for account in accounts {
      assert_ok!(Adapter::mint_into(asset, &account, amount));
    }
  }

  fn create_tdfy_to_temp_limit_swap_request(
    &self,
    requester_account_id: AccountId,
    tdfy_amount: Balance,
    temp_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      CurrencyId::Tdfy,
      tdfy_amount,
      TEMP_CURRENCY_ID,
      temp_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      MarketMakers::<Test>::get(&requester_account_id).is_some(),
      SwapType::Limit,
      slippage,
    )
  }

  fn create_temp_to_tdfy_limit_swap_request(
    &self,
    requester_account_id: AccountId,
    temp_amount: Balance,
    tdfy_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      TEMP_CURRENCY_ID,
      temp_amount,
      CurrencyId::Tdfy,
      tdfy_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      MarketMakers::<Test>::get(&requester_account_id).is_some(),
      SwapType::Limit,
      slippage,
    )
  }

  fn create_temp_to_zemp_limit_swap_request(
    &self,
    requester_account_id: AccountId,
    temp_amount: Balance,
    zemp_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      TEMP_CURRENCY_ID,
      temp_amount,
      ZEMP_CURRENCY_ID,
      zemp_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      MarketMakers::<Test>::get(&requester_account_id).is_some(),
      SwapType::Limit,
      slippage,
    )
  }

  fn create_tdfy_to_temp_market_swap_request(
    &self,
    requester_account_id: AccountId,
    tdfy_amount: Balance,
    temp_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      CurrencyId::Tdfy,
      tdfy_amount,
      TEMP_CURRENCY_ID,
      temp_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      MarketMakers::<Test>::get(&requester_account_id).is_some(),
      SwapType::Market,
      slippage,
    )
  }

  fn create_temp_to_tdfy_market_swap_request(
    &self,
    requester_account_id: AccountId,
    temp_amount: Balance,
    tdfy_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      TEMP_CURRENCY_ID,
      temp_amount,
      CurrencyId::Tdfy,
      tdfy_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      MarketMakers::<Test>::get(&requester_account_id).is_some(),
      SwapType::Market,
      slippage,
    )
  }

  fn create_zemp_to_temp_market_swap_request(
    &self,
    requester_account_id: AccountId,
    zemp_amount: Balance,
    temp_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      ZEMP_CURRENCY_ID,
      zemp_amount,
      TEMP_CURRENCY_ID,
      temp_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      MarketMakers::<Test>::get(&requester_account_id).is_some(),
      SwapType::Market,
      slippage,
    )
  }

  fn create_zemp_to_temp_limit_swap_request(
    &self,
    requester_account_id: AccountId,
    zemp_amount: Balance,
    temp_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      ZEMP_CURRENCY_ID,
      zemp_amount,
      TEMP_CURRENCY_ID,
      temp_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      MarketMakers::<Test>::get(&requester_account_id).is_some(),
      SwapType::Limit,
      slippage,
    )
  }

  fn create_temp_to_zemp_market_swap_request(
    &self,
    requester_account_id: AccountId,
    temp_amount: Balance,
    zemp_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      TEMP_CURRENCY_ID,
      temp_amount,
      ZEMP_CURRENCY_ID,
      zemp_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      MarketMakers::<Test>::get(&requester_account_id).is_some(),
      SwapType::Market,
      slippage,
    )
  }
}

fn get_account_reserved(account_id: AccountId, currency_id: CurrencyId) -> Balance {
  match currency_id {
    CurrencyId::Tdfy => Balances::reserved_balance(account_id),
    CurrencyId::Wrapped(asset_id) => Account::<Test>::get(account_id, asset_id).unwrap().reserved,
  }
}

fn swap_exists_with_status(request_id: Hash, status: SwapStatus) -> Swap<AccountId, u64> {
  let swap = Oracle::swaps(request_id);
  assert!(swap.is_some());
  let swap = swap.unwrap();
  assert_eq!(status, swap.status);
  swap
}

fn account_swap_exists_with_status(account: AccountId, request_id: Hash, status: SwapStatus) {
  let account_swaps = Oracle::account_swaps(account);
  assert!(account_swaps.is_some());
  let account_swaps = account_swaps.unwrap();
  let (_, account_swap_status) = account_swaps
    .iter()
    .find(|(account_swap_request_id, _)| *account_swap_request_id == request_id)
    .unwrap();
  assert_eq!(status, *account_swap_status);
}

fn account_swap_is_deleted(account: AccountId, request_id: Hash) {
  let account_swaps = Oracle::account_swaps(account).unwrap();
  assert_eq!(
    account_swaps
      .iter()
      .find(|(account_swap_request_id, _)| *account_swap_request_id == request_id),
    None
  );
}

#[allow(clippy::too_many_arguments)]
fn add_new_swap_and_assert_results(
  account_id: AccountId,
  asset_id_from: CurrencyId,
  amount_from: Balance,
  asset_id_to: CurrencyId,
  amount_to: Balance,
  block_number: BlockNumber,
  extrinsic_hash: [u8; 32],
  is_market_maker: bool,
  swap_type: SwapType,
  slippage: Permill,
) -> Hash {
  let (trade_request_id, trade_request) = Oracle::add_new_swap_in_queue(
    account_id,
    asset_id_from,
    amount_from,
    asset_id_to,
    amount_to,
    block_number,
    extrinsic_hash,
    is_market_maker,
    swap_type.clone(),
    slippage,
  )
  .unwrap();

  assert_eq!(trade_request.status, SwapStatus::Pending);
  account_swap_exists_with_status(account_id, trade_request_id, SwapStatus::Pending);
  assert_eq!(trade_request.block_number, CURRENT_BLOCK_NUMBER);

  trade_request_id
}

fn assert_spendable_balance_is_updated(
  account_id: AccountId,
  currency_id: CurrencyId,
  initial_balance: Balance,
  sell_amount: Balance,
  swap_fee: Balance,
) {
  let expected_reducible_balance = initial_balance
    .saturating_sub(sell_amount)
    .saturating_sub(swap_fee);

  match currency_id {
    CurrencyId::Tdfy => assert_eq!(
      Adapter::reducible_balance(currency_id, &account_id, true),
      expected_reducible_balance
    ),
    _ => assert_eq!(
      Adapter::reducible_balance(currency_id, &account_id, true),
      expected_reducible_balance.saturating_sub(1_u128) // keep-alive token
    ),
  }

  assert_eq!(
    Adapter::reducible_balance(currency_id, &account_id, false),
    initial_balance
      .saturating_sub(sell_amount)
      .saturating_sub(swap_fee)
  );
}

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    assert!(!Oracle::status());
  });
}

#[test]
pub fn set_operational_status_works() {
  new_test_ext().execute_with(|| {
    let context = Context::default();

    assert_ok!(Oracle::set_status(context.alice.clone(), true));
    assert_noop!(
      Oracle::set_status(context.bob, false),
      Error::<Test>::AccessDenied
    );
    assert!(Oracle::status());
    assert_ok!(Oracle::set_status(context.alice, false));
    assert!(!Oracle::status());
  });
}

#[test]
pub fn test_slippage() {
  new_test_ext().execute_with(|| {
    const INITIAL_100_TEMPS: Balance = 100 * ONE_TEMP;

    let context = Context::default()
      .set_oracle_status(true)
      .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
      .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
      .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
      .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS * 5)
      .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_100_TEMPS * 2);

    assert_eq!(
      Adapter::balance_on_hold(TEMP_CURRENCY_ID, &CHARLIE_ACCOUNT_ID),
      0
    );

    let trade_request_id = context.create_tdfy_to_temp_market_swap_request(
      BOB_ACCOUNT_ID,
      INITIAL_20_TDFYS,
      INITIAL_100_TEMPS,
      EXTRINSIC_HASH_0,
      SLIPPAGE_2_PERCENTS,
    );

    assert_eq!(
      Adapter::balance_on_hold(TEMP_CURRENCY_ID, &CHARLIE_ACCOUNT_ID),
      0
    );

    assert_eq!(
      trade_request_id,
      Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
        .unwrap_or_default()
    );

    let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
      CHARLIE_ACCOUNT_ID,
      // ratio is a bit different (mm is willing to pay a bit more for the same amount)
      INITIAL_100_TEMPS,
      INITIAL_20_TDFYS,
      EXTRINSIC_HASH_0,
      SLIPPAGE_0_PERCENT,
    );

    assert_eq!(
      trade_request_mm_id,
      Hash::from_str("0x9ee76e89d3eae9ddad2e0b731e29ddcfa0781f7035600c5eb885637592e1d2c2")
        .unwrap_or_default()
    );

    assert_noop!(
      Oracle::confirm_swap(
        context.alice.clone(),
        trade_request_id,
        vec![SwapConfirmation {
          request_id: trade_request_mm_id,
          amount_to_receive: INITIAL_20_TDFYS.saturating_div(2),
          amount_to_send: INITIAL_100_TEMPS
            .saturating_div(2)
            .saturating_sub(ONE_TEMP * 2),
        },],
      ),
      Error::<Test>::OfferIsLessThanSwapLowerBound { index: 0 }
    );

    assert_noop!(
      Oracle::confirm_swap(
        context.alice.clone(),
        trade_request_id,
        vec![SwapConfirmation {
          request_id: trade_request_mm_id,
          amount_to_receive: INITIAL_20_TDFYS.saturating_div(2),
          amount_to_send: INITIAL_100_TEMPS
            .saturating_div(2)
            .saturating_add(ONE_TEMP * 2),
        },],
      ),
      Error::<Test>::OfferIsGreaterThanMarketMakerSwapUpperBound { index: 0 }
    );

    // partial filling
    assert_ok!(Oracle::confirm_swap(
      context.alice,
      trade_request_id,
      vec![SwapConfirmation {
        request_id: trade_request_mm_id,
        amount_to_receive: INITIAL_20_TDFYS.saturating_div(2),
        amount_to_send: INITIAL_100_TEMPS.saturating_div(2),
      },],
    ));

    // market order got deleted
    assert!(Oracle::swaps(trade_request_id).is_none());
    // limit order isnt deleted as its not fully filled
    swap_exists_with_status(trade_request_mm_id, SwapStatus::PartiallyFilled);
  });
}

#[test]
pub fn test_imalive() {
  new_test_ext().execute_with(|| {
    let context = Context::default()
      .set_oracle_status(true)
      .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY);

    assert_ok!(Oracle::update_assets_value(
      context.alice,
      vec![
        // 10 Tdfy / USDT
        (4, 10_000_000_000_000_u128),
        // 100k Tdfy / BTC
        (2, 100_000_000_000_000_000_u128),
      ]
    ));

    let fee =
      Fees::calculate_swap_fees(CurrencyId::Wrapped(4), 100_000_000, SwapType::Limit, false);
    assert_eq!(
      Sunrise::calculate_rebates_on_fees_paid(
        // 125%
        FixedU128::saturating_from_rational(125, 100),
        // 2$ USDT in fee
        // Should have total 2.5$ USDT in reward
        // 2.5 / 0.1 = 25 TDFY final
        &fee,
      )
      .unwrap(),
      25_000_000_000_000
    );
  });
}

mod confirm_swap {
  use super::*;

  const CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS: Balance = 100 * ONE_TEMP;
  const CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS: Balance = 5 * ONE_TDFY;

  fn create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
    context: &Context,
  ) -> Hash {
    context.create_tdfy_to_temp_limit_swap_request(
      BOB_ACCOUNT_ID,
      BOB_SELLS_10_TDFYS,
      BOB_BUYS_200_TEMPS,
      EXTRINSIC_HASH_0,
      SLIPPAGE_2_PERCENTS,
    )
  }

  fn create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
    context: &Context,
  ) -> Hash {
    context.create_temp_to_tdfy_limit_swap_request(
      CHARLIE_ACCOUNT_ID,
      CHARLIE_SELLS_4000_TEMPS,
      CHARLIE_BUYS_200_TDFYS,
      EXTRINSIC_HASH_1,
      SLIPPAGE_4_PERCENTS,
    )
  }

  mod succeed_when {
    use super::*;

    #[test]
    fn offer_full_filled_market_makers() {
      new_test_ext().execute_with(|| {
        Context::default()
          .set_oracle_status(true)
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(DAVE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS)
          .mint_temp(DAVE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let market_maker_sets = vec![
          vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID],
          vec![BOB_ACCOUNT_ID, CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID],
        ];

        for market_makers in market_maker_sets.iter() {
          let context: Context = Context::default().set_market_makers((*market_makers).clone());

          let bob_initial_tdfy_balance = Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID);
          let charlie_initial_tdfy_balance =
            Adapter::balance(CurrencyId::Tdfy, &CHARLIE_ACCOUNT_ID);
          let dave_initial_tdfy_balance = Adapter::balance(CurrencyId::Tdfy, &DAVE_ACCOUNT_ID);
          let fees_account_initial_tdfy_balance =
            Adapter::balance(CurrencyId::Tdfy, &Fees::account_id());

          let bob_initial_temp_balance = Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID);
          let charlie_initial_temp_balance =
            Adapter::balance(TEMP_CURRENCY_ID, &CHARLIE_ACCOUNT_ID);
          let dave_initial_temp_balance = Adapter::balance(TEMP_CURRENCY_ID, &DAVE_ACCOUNT_ID);
          let fees_account_initial_temp_balance =
            Adapter::balance(TEMP_CURRENCY_ID, &Fees::account_id());

          // Create a trader swap
          let trade_request_id = context.create_tdfy_to_temp_market_swap_request(
            BOB_ACCOUNT_ID,
            BOB_SELLS_10_TDFYS,
            BOB_BUYS_200_TEMPS,
            EXTRINSIC_HASH_0,
            SLIPPAGE_5_PERCENTS,
          );

          // Create 2 market maker swaps
          let trade_request_mm_id_1 = context.create_temp_to_tdfy_limit_swap_request(
            CHARLIE_ACCOUNT_ID,
            CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
            EXTRINSIC_HASH_1,
            SLIPPAGE_4_PERCENTS,
          );
          let trade_request_mm_id_2 = context.create_temp_to_tdfy_limit_swap_request(
            DAVE_ACCOUNT_ID,
            DAVE_PARTIAL_FILLING_SELLS_60_TEMPS,
            DAVE_PARTIAL_FILLING_BUYS_3_TDFYS,
            EXTRINSIC_HASH_2,
            SLIPPAGE_2_PERCENTS,
          );

          // Call confirm_swaps with 2 valid market maker swap confirmations
          assert_ok!(Oracle::confirm_swap(
            RuntimeOrigin::signed(ALICE_ACCOUNT_ID),
            trade_request_id,
            vec![
              SwapConfirmation {
                request_id: trade_request_mm_id_1,
                amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
                amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
              },
              SwapConfirmation {
                request_id: trade_request_mm_id_2,
                amount_to_receive: DAVE_PARTIAL_FILLING_BUYS_3_TDFYS,
                amount_to_send: DAVE_PARTIAL_FILLING_SELLS_60_TEMPS,
              },
            ],
          ));

          let total_swapped_tdfys =
            CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS + DAVE_PARTIAL_FILLING_BUYS_3_TDFYS;
          let trader_swap_fees = Fees::calculate_swap_fees(
            CurrencyId::Tdfy,
            total_swapped_tdfys,
            SwapType::Market,
            market_makers.contains(&BOB_ACCOUNT_ID),
          )
          .fee;
          let charlie_swap_fee = Fees::calculate_swap_fees(
            TEMP_CURRENCY_ID,
            CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            SwapType::Limit,
            market_makers.contains(&CHARLIE_ACCOUNT_ID),
          )
          .fee;
          let dave_swap_fee = Fees::calculate_swap_fees(
            TEMP_CURRENCY_ID,
            DAVE_PARTIAL_FILLING_SELLS_60_TEMPS,
            SwapType::Limit,
            market_makers.contains(&DAVE_ACCOUNT_ID),
          )
          .fee;

          // Events are emitted
          System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
            request_id: trade_request_id,
            status: SwapStatus::PartiallyFilled,
            account_id: BOB_ACCOUNT_ID,
            currency_from: CurrencyId::Tdfy,
            currency_amount_from: total_swapped_tdfys,
            currency_to: TEMP_CURRENCY_ID,
            currency_amount_to: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
              + DAVE_PARTIAL_FILLING_SELLS_60_TEMPS,
            initial_extrinsic_hash: EXTRINSIC_HASH_0,
          }));

          System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
            request_id: trade_request_mm_id_1,
            status: SwapStatus::Completed,
            account_id: CHARLIE_ACCOUNT_ID,
            currency_from: TEMP_CURRENCY_ID,
            currency_amount_from: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            currency_to: CurrencyId::Tdfy,
            currency_amount_to: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
            initial_extrinsic_hash: EXTRINSIC_HASH_1,
          }));

          System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
            request_id: trade_request_mm_id_2,
            status: SwapStatus::Completed,
            account_id: DAVE_ACCOUNT_ID,
            currency_from: TEMP_CURRENCY_ID,
            currency_amount_from: DAVE_PARTIAL_FILLING_SELLS_60_TEMPS,
            currency_to: CurrencyId::Tdfy,
            currency_amount_to: DAVE_PARTIAL_FILLING_BUYS_3_TDFYS,
            initial_extrinsic_hash: EXTRINSIC_HASH_2,
          }));

          assert_eq!(
            Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
            bob_initial_tdfy_balance - total_swapped_tdfys - trader_swap_fees
          );
          assert_eq!(
            Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
            bob_initial_temp_balance
              + CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
              + DAVE_PARTIAL_FILLING_SELLS_60_TEMPS
          );

          assert_eq!(
            Adapter::balance(CurrencyId::Tdfy, &CHARLIE_ACCOUNT_ID),
            charlie_initial_tdfy_balance + CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS
          );
          assert_eq!(
            Adapter::balance(TEMP_CURRENCY_ID, &CHARLIE_ACCOUNT_ID),
            charlie_initial_temp_balance
              - CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
              - charlie_swap_fee
          );

          assert_eq!(
            Adapter::balance(CurrencyId::Tdfy, &DAVE_ACCOUNT_ID),
            dave_initial_tdfy_balance + DAVE_PARTIAL_FILLING_BUYS_3_TDFYS
          );
          assert_eq!(
            Adapter::balance(TEMP_CURRENCY_ID, &DAVE_ACCOUNT_ID),
            dave_initial_temp_balance - DAVE_PARTIAL_FILLING_SELLS_60_TEMPS - dave_swap_fee
          );

          assert_eq!(
            Adapter::balance(CurrencyId::Tdfy, &Fees::account_id()),
            fees_account_initial_tdfy_balance + trader_swap_fees // Fees account has 1 TDFY, which is an existential deposit initially
          );
          assert_eq!(
            Adapter::balance(TEMP_CURRENCY_ID, &Fees::account_id()),
            fees_account_initial_temp_balance + charlie_swap_fee + dave_swap_fee
          );

          // Trader market swap is removed from Swaps
          assert!(Oracle::swaps(trade_request_id).is_none());

          // Trader market swap is removed from AccountSwaps
          account_swap_is_deleted(BOB_ACCOUNT_ID, trade_request_id);

          // Charlie limit swap is removed from Swaps
          assert!(Oracle::swaps(trade_request_mm_id_1).is_none());

          // Charlie limit swap is removed from AccountSwaps
          account_swap_is_deleted(CHARLIE_ACCOUNT_ID, trade_request_mm_id_1);

          // Dave limit swap is removed from Swaps
          assert!(Oracle::swaps(trade_request_mm_id_2).is_none());

          // Dave limit swap is removed from AccountSwaps
          account_swap_is_deleted(DAVE_ACCOUNT_ID, trade_request_mm_id_2);

          // Trader's reserved fund is fully released as its swap type is market swap
          assert_eq!(get_account_reserved(BOB_ACCOUNT_ID, CurrencyId::Tdfy), 0);

          // Bob's reserved fund is fully released as its swap is completed
          assert_eq!(get_account_reserved(BOB_ACCOUNT_ID, TEMP_CURRENCY_ID), 0);

          // Charlie's reserved fund is fully released as its swap is completed
          assert_eq!(
            get_account_reserved(CHARLIE_ACCOUNT_ID, TEMP_CURRENCY_ID),
            0
          );

          // Dave's reserved fund is fully released as its swap is completed
          assert_eq!(get_account_reserved(DAVE_ACCOUNT_ID, TEMP_CURRENCY_ID), 0);
        }
      });
    }

    #[test]
    fn offer_is_full_filled_by_market_makers() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(DAVE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS)
          .mint_temp(DAVE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        Fees::start_era();
        assert!(Fees::current_era().is_some());
        let current_era = Fees::current_era().unwrap().index;

        let trade_request_id = context.create_tdfy_to_temp_limit_swap_request(
          BOB_ACCOUNT_ID,
          BOB_SELLS_10_TDFYS,
          BOB_BUYS_200_TEMPS,
          EXTRINSIC_HASH_0,
          SLIPPAGE_2_PERCENTS,
        );

        assert_eq!(
          trade_request_id,
          Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
            .unwrap_or_default()
        );

        let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
          CHARLIE_ACCOUNT_ID,
          CHARLIE_SELLS_4000_TEMPS,
          CHARLIE_BUYS_200_TDFYS,
          EXTRINSIC_HASH_1,
          SLIPPAGE_5_PERCENTS,
        );

        assert_eq!(
          trade_request_mm_id,
          Hash::from_str("0x9ee76e89d3eae9ddad2e0b731e29ddcfa0781f7035600c5eb885637592e1d2c2")
            .unwrap_or_default()
        );

        const DAVE_SELLS_100_TEMPS: Balance = 100 * ONE_TEMP;
        const DAVE_BUYS_5_TDFYS: Balance = 5 * ONE_TDFY;
        let trade_request_mm2_id = context.create_temp_to_tdfy_limit_swap_request(
          DAVE_ACCOUNT_ID,
          DAVE_SELLS_100_TEMPS,
          DAVE_BUYS_5_TDFYS,
          EXTRINSIC_HASH_2,
          SLIPPAGE_4_PERCENTS,
        );

        // partial fillings
        const CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS: Balance = 5 * ONE_TDFY;
        const CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS: Balance = 100 * ONE_TEMP;
        const DAVE_PARTIAL_FILLING_BUYS_5_TDFYS: Balance = 5 * ONE_TDFY;
        const DAVE_PARTIAL_FILLING_SELLS_100_TEMPS: Balance = 100 * ONE_TEMP;

        assert_ok!(Oracle::confirm_swap(
          context.alice,
          trade_request_id,
          vec![
            SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },
            SwapConfirmation {
              request_id: trade_request_mm2_id,
              amount_to_receive: DAVE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: DAVE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },
          ],
        ));

        // swap confirmation for bob (user)
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_id,
          status: SwapStatus::Completed,
          account_id: BOB_ACCOUNT_ID,
          currency_from: CurrencyId::Tdfy,
          currency_amount_from: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS
            + DAVE_PARTIAL_FILLING_BUYS_5_TDFYS,
          currency_to: TEMP_CURRENCY_ID,
          currency_amount_to: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
            + DAVE_PARTIAL_FILLING_SELLS_100_TEMPS,
          initial_extrinsic_hash: EXTRINSIC_HASH_0,
        }));

        // swap confirmation for charlie (mm1)
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_mm_id,
          status: SwapStatus::PartiallyFilled,
          account_id: CHARLIE_ACCOUNT_ID,
          currency_from: TEMP_CURRENCY_ID,
          currency_amount_from: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
          currency_to: CurrencyId::Tdfy,
          currency_amount_to: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
          initial_extrinsic_hash: EXTRINSIC_HASH_1,
        }));

        // swap confirmation for dave (mm2)
        // the trade should be closed, because amount_from of the request is filled
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_mm2_id,
          status: SwapStatus::Completed,
          account_id: DAVE_ACCOUNT_ID,
          currency_from: TEMP_CURRENCY_ID,
          currency_amount_from: DAVE_PARTIAL_FILLING_SELLS_100_TEMPS,
          currency_to: CurrencyId::Tdfy,
          currency_amount_to: DAVE_PARTIAL_FILLING_BUYS_5_TDFYS,
          initial_extrinsic_hash: EXTRINSIC_HASH_2,
        }));

        // BOB: make sure the CLIENT current trade is deleted
        assert!(Oracle::swaps(trade_request_id).is_none());
        account_swap_is_deleted(BOB_ACCOUNT_ID, trade_request_id);

        // CHARLIE: make sure the MM current trade is partially filled and correctly updated
        account_swap_exists_with_status(
          CHARLIE_ACCOUNT_ID,
          trade_request_mm_id,
          SwapStatus::PartiallyFilled,
        );

        let trade_request_filled =
          swap_exists_with_status(trade_request_mm_id, SwapStatus::PartiallyFilled);
        assert_eq!(
          trade_request_filled.amount_from_filled,
          CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
        );
        assert_eq!(
          trade_request_filled.amount_to_filled,
          CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS
        );

        // DAVE: make sure the MM current trade is totally filled (deleted)
        assert!(Oracle::swaps(trade_request_mm2_id).is_none());
        account_swap_is_deleted(DAVE_ACCOUNT_ID, trade_request_mm2_id);

        // make sure all balances match
        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &context.fees_account_id),
          // we burned 1 tdfy on start so it should contain 1.2 tdfy now
          ONE_TDFY + REQUESTER_SWAP_FEE_RATE * BOB_SELLS_10_TDFYS
        );

        assert_eq!(
          Adapter::balance(TEMP_CURRENCY_ID, &context.fees_account_id),
          MARKET_MAKER_SWAP_FEE_RATE
            * (CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS + DAVE_PARTIAL_FILLING_SELLS_100_TEMPS)
        );

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
          INITIAL_20_TDFYS
            .saturating_sub(BOB_SELLS_10_TDFYS)
            .saturating_sub(REQUESTER_SWAP_FEE_RATE * BOB_SELLS_10_TDFYS)
        );

        assert_eq!(
          Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
          BOB_BUYS_200_TEMPS
        );

        // make sure fees are registered on chain
        let bob_fee = Fees::account_fees(current_era, BOB_ACCOUNT_ID);
        assert_eq!(
          bob_fee.first().unwrap().1.fee,
          REQUESTER_SWAP_FEE_RATE * BOB_SELLS_10_TDFYS
        );
        assert_eq!(bob_fee.first().unwrap().1.amount, BOB_SELLS_10_TDFYS);

        let charlie_fee = Fees::account_fees(current_era, CHARLIE_ACCOUNT_ID);
        assert_eq!(
          charlie_fee.first().unwrap().1.fee,
          MARKET_MAKER_SWAP_FEE_RATE * CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
        );
        assert_eq!(
          charlie_fee.first().unwrap().1.amount,
          CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
        );

        let dave_fee = Fees::account_fees(current_era, DAVE_ACCOUNT_ID);
        assert_eq!(
          dave_fee.first().unwrap().1.fee,
          MARKET_MAKER_SWAP_FEE_RATE * DAVE_PARTIAL_FILLING_SELLS_100_TEMPS
        );
        assert_eq!(
          dave_fee.first().unwrap().1.amount,
          DAVE_PARTIAL_FILLING_SELLS_100_TEMPS
        );
      });
    }

    #[test]
    fn partial_filling_limit_trade_multiple_times() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(DAVE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS)
          .mint_temp(DAVE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id = context.create_tdfy_to_temp_limit_swap_request(
          BOB_ACCOUNT_ID,
          BOB_SELLS_10_TDFYS,
          BOB_BUYS_200_TEMPS,
          EXTRINSIC_HASH_0,
          SLIPPAGE_2_PERCENTS,
        );

        assert_eq!(
          trade_request_id,
          Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
            .unwrap_or_default()
        );

        let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
          CHARLIE_ACCOUNT_ID,
          CHARLIE_SELLS_4000_TEMPS,
          CHARLIE_BUYS_200_TDFYS,
          EXTRINSIC_HASH_1,
          SLIPPAGE_4_PERCENTS,
        );

        assert_eq!(
          trade_request_mm_id,
          Hash::from_str("0x9ee76e89d3eae9ddad2e0b731e29ddcfa0781f7035600c5eb885637592e1d2c2")
            .unwrap_or_default()
        );

        const DAVE_SELLS_8000_TEMPS: Balance = 8_000 * ONE_TEMP;
        const DAVE_BUYS_400_TDFYS: Balance = 400 * ONE_TDFY;
        let trade_request_mm2_id = context.create_temp_to_tdfy_limit_swap_request(
          DAVE_ACCOUNT_ID,
          DAVE_SELLS_8000_TEMPS,
          DAVE_BUYS_400_TDFYS,
          EXTRINSIC_HASH_2,
          SLIPPAGE_5_PERCENTS,
        );

        const CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS: Balance = 100 * ONE_TEMP;
        const CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS: Balance = 5 * ONE_TDFY;
        // partial filling
        assert_ok!(Oracle::confirm_swap(
          context.alice.clone(),
          trade_request_id,
          vec![
            // charlie
            SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },
          ],
        ));

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
          INITIAL_20_TDFYS
            .saturating_sub(CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS)
            .saturating_sub(REQUESTER_SWAP_FEE_RATE * CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS)
        );

        // swap confirmation for bob (user)
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_id,
          status: SwapStatus::PartiallyFilled,
          account_id: BOB_ACCOUNT_ID,
          currency_from: CurrencyId::Tdfy,
          currency_amount_from: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
          currency_to: TEMP_CURRENCY_ID,
          currency_amount_to: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
          initial_extrinsic_hash: EXTRINSIC_HASH_0,
        }));

        // swap confirmation for charlie (mm)
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_mm_id,
          status: SwapStatus::PartiallyFilled,
          account_id: CHARLIE_ACCOUNT_ID,
          currency_from: TEMP_CURRENCY_ID,
          currency_amount_from: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
          currency_to: CurrencyId::Tdfy,
          currency_amount_to: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
          initial_extrinsic_hash: EXTRINSIC_HASH_1,
        }));

        // BOB: make sure the CLIENT current trade is partially filled and correctly updated
        let trade_request_filled =
          swap_exists_with_status(trade_request_id, SwapStatus::PartiallyFilled);
        account_swap_exists_with_status(
          BOB_ACCOUNT_ID,
          trade_request_id,
          SwapStatus::PartiallyFilled,
        );

        assert_eq!(
          trade_request_filled.amount_from_filled,
          CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS
        );
        assert_eq!(
          trade_request_filled.amount_to_filled,
          CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
        );

        // CHARLIE: make sure the MM current trade is partially filled and correctly updated
        let charlie_trade_request_filled =
          swap_exists_with_status(trade_request_mm_id, SwapStatus::PartiallyFilled);
        assert_eq!(
          charlie_trade_request_filled.amount_from_filled,
          CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
        );
        assert_eq!(
          charlie_trade_request_filled.amount_to_filled,
          CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS
        );

        const DAVE_PARTIAL_FILLING_SELLS_100_TEMPS: Balance = 100 * ONE_TEMP;
        const DAVE_PARTIAL_FILLING_BUYS_5_TDFYS: Balance = 5 * ONE_TDFY;

        // another partial filling who should close the trade
        assert_ok!(Oracle::confirm_swap(
          context.alice.clone(),
          trade_request_id,
          vec![SwapConfirmation {
            request_id: trade_request_mm2_id,
            amount_to_receive: DAVE_PARTIAL_FILLING_BUYS_5_TDFYS,
            amount_to_send: DAVE_PARTIAL_FILLING_SELLS_100_TEMPS,
          },],
        ));

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
          INITIAL_20_TDFYS
            .saturating_sub(10 * ONE_TDFY)
            .saturating_sub(REQUESTER_SWAP_FEE_RATE * (10 * ONE_TDFY))
        );

        // swap confirmation for bob (user)
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_id,
          status: SwapStatus::Completed,
          account_id: BOB_ACCOUNT_ID,
          currency_from: CurrencyId::Tdfy,
          currency_amount_from: DAVE_PARTIAL_FILLING_BUYS_5_TDFYS,
          currency_to: TEMP_CURRENCY_ID,
          currency_amount_to: DAVE_PARTIAL_FILLING_SELLS_100_TEMPS,
          initial_extrinsic_hash: EXTRINSIC_HASH_0,
        }));

        // swap confirmation for dave (second mm)
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_mm2_id,
          status: SwapStatus::PartiallyFilled,
          account_id: DAVE_ACCOUNT_ID,
          currency_from: TEMP_CURRENCY_ID,
          currency_amount_from: DAVE_PARTIAL_FILLING_SELLS_100_TEMPS,
          currency_to: CurrencyId::Tdfy,
          currency_amount_to: DAVE_PARTIAL_FILLING_BUYS_5_TDFYS,
          initial_extrinsic_hash: EXTRINSIC_HASH_2,
        }));

        // BOB: make sure the CLIENT current trade is deleted
        assert!(Oracle::swaps(trade_request_id).is_none());
        account_swap_is_deleted(BOB_ACCOUNT_ID, trade_request_id);

        // cant send another trade confirmation as the request should be deleted
        // we do expect `InvalidRequestId`
        assert_noop!(
          Oracle::confirm_swap(context.alice.clone(), trade_request_id, vec![],),
          Error::<Test>::InvalidRequestId
        );

        // DAVE: make sure the MM current trade is partially filled and correctly updated
        let dave_trade_request_filled =
          swap_exists_with_status(trade_request_mm2_id, SwapStatus::PartiallyFilled);
        assert_eq!(
          dave_trade_request_filled.amount_from_filled,
          DAVE_PARTIAL_FILLING_SELLS_100_TEMPS
        );
        assert_eq!(
          dave_trade_request_filled.amount_to_filled,
          DAVE_PARTIAL_FILLING_BUYS_5_TDFYS
        );

        // cancel our mm's swap to release the funds
        assert_ok!(Oracle::cancel_swap(
          context.alice.clone(),
          trade_request_mm_id,
        ));
        assert_ok!(Oracle::cancel_swap(context.alice, trade_request_mm2_id,));

        // validate all balance
        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
          INITIAL_20_TDFYS
            .saturating_sub(BOB_SELLS_10_TDFYS)
            .saturating_sub(REQUESTER_SWAP_FEE_RATE * BOB_SELLS_10_TDFYS)
        );
        assert_eq!(
          Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
          BOB_BUYS_200_TEMPS
        );
        assert_eq!(
          Adapter::balance_on_hold(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
          Zero::zero()
        );

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &CHARLIE_ACCOUNT_ID),
          // initial balance + swap
          ONE_TDFY + CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS
        );

        let swap_fee = Fees::calculate_swap_fees(
          TEMP_CURRENCY_ID,
          CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
          SwapType::Limit,
          true,
        )
        .fee;

        assert_spendable_balance_is_updated(
          CHARLIE_ACCOUNT_ID,
          TEMP_CURRENCY_ID,
          INITIAL_10000_TEMPS,
          CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
          swap_fee,
        );

        assert_eq!(
          Adapter::balance_on_hold(TEMP_CURRENCY_ID, &CHARLIE_ACCOUNT_ID),
          Zero::zero()
        );

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &DAVE_ACCOUNT_ID),
          // initial balance + swap
          ONE_TDFY + DAVE_PARTIAL_FILLING_BUYS_5_TDFYS
        );

        assert_eq!(
          Adapter::balance(TEMP_CURRENCY_ID, &DAVE_ACCOUNT_ID),
          INITIAL_10000_TEMPS
            .saturating_sub(DAVE_PARTIAL_FILLING_SELLS_100_TEMPS)
            .saturating_sub(MARKET_MAKER_SWAP_FEE_RATE * DAVE_PARTIAL_FILLING_SELLS_100_TEMPS)
        );

        assert_eq!(
          Adapter::balance_on_hold(TEMP_CURRENCY_ID, &DAVE_ACCOUNT_ID),
          Zero::zero()
        );
      });
    }

    #[test]
    fn market_trade_can_only_be_filled_once() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(DAVE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS)
          .mint_temp(DAVE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id = context.create_tdfy_to_temp_market_swap_request(
          BOB_ACCOUNT_ID,
          BOB_SELLS_10_TDFYS,
          BOB_BUYS_200_TEMPS,
          EXTRINSIC_HASH_0,
          SLIPPAGE_2_PERCENTS,
        );

        assert_eq!(
          trade_request_id,
          Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
            .unwrap_or_default()
        );

        let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
          CHARLIE_ACCOUNT_ID,
          CHARLIE_SELLS_4000_TEMPS,
          CHARLIE_BUYS_200_TDFYS,
          EXTRINSIC_HASH_1,
          SLIPPAGE_4_PERCENTS,
        );

        assert_eq!(
          trade_request_mm_id,
          Hash::from_str("0x9ee76e89d3eae9ddad2e0b731e29ddcfa0781f7035600c5eb885637592e1d2c2")
            .unwrap_or_default()
        );

        const DAVE_SELLS_8000_TEMPS: Balance = 8_000 * ONE_TEMP;
        const DAVE_BUYS_400_TDFYS: Balance = 400 * ONE_TDFY;
        let trade_request_mm2_id = context.create_temp_to_tdfy_limit_swap_request(
          DAVE_ACCOUNT_ID,
          DAVE_SELLS_8000_TEMPS,
          DAVE_BUYS_400_TDFYS,
          EXTRINSIC_HASH_2,
          SLIPPAGE_5_PERCENTS,
        );

        const CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS: Balance = 100 * ONE_TEMP;
        const CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS: Balance = 5 * ONE_TDFY;
        // partial filling
        assert_ok!(Oracle::confirm_swap(
          context.alice.clone(),
          trade_request_id,
          vec![
            // charlie
            SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },
          ],
        ));

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
          INITIAL_20_TDFYS
            .saturating_sub(CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS)
            .saturating_sub(REQUESTER_SWAP_FEE_RATE * CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS)
        );

        // swap confirmation for bob (user)
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_id,
          status: SwapStatus::PartiallyFilled,
          account_id: BOB_ACCOUNT_ID,
          currency_from: CurrencyId::Tdfy,
          currency_amount_from: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
          currency_to: TEMP_CURRENCY_ID,
          currency_amount_to: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
          initial_extrinsic_hash: EXTRINSIC_HASH_0,
        }));

        // swap confirmation for charlie (mm)
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_mm_id,
          status: SwapStatus::PartiallyFilled,
          account_id: CHARLIE_ACCOUNT_ID,
          currency_from: TEMP_CURRENCY_ID,
          currency_amount_from: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
          currency_to: CurrencyId::Tdfy,
          currency_amount_to: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
          initial_extrinsic_hash: EXTRINSIC_HASH_1,
        }));

        // BOB: make sure the CLIENT current trade is deleted
        assert!(Oracle::swaps(trade_request_id).is_none());
        account_swap_is_deleted(BOB_ACCOUNT_ID, trade_request_id);

        // CHARLIE: make sure the MM current trade is partially filled and correctly updated
        let charlie_trade_request_filled =
          swap_exists_with_status(trade_request_mm_id, SwapStatus::PartiallyFilled);
        assert_eq!(
          charlie_trade_request_filled.amount_from_filled,
          CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
        );
        assert_eq!(
          charlie_trade_request_filled.amount_to_filled,
          CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS
        );

        const DAVE_PARTIAL_FILLING_SELLS_100_TEMPS: Balance = 100 * ONE_TEMP;
        const DAVE_PARTIAL_FILLING_BUYS_5_TDFYS: Balance = 5 * ONE_TDFY;

        // cant send another trade confirmation as the request should be deleted
        // we do expect `InvalidRequestId`
        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm2_id,
              amount_to_receive: DAVE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: DAVE_PARTIAL_FILLING_SELLS_100_TEMPS,
            }],
          ),
          Error::<Test>::InvalidRequestId
        );

        // DAVE: make sure the MM current trade is still pending
        let dave_trade_request_filled =
          swap_exists_with_status(trade_request_mm2_id, SwapStatus::Pending);
        assert_eq!(dave_trade_request_filled.amount_from_filled, Zero::zero());
        assert_eq!(dave_trade_request_filled.amount_to_filled, Zero::zero());

        // cancel our mm's swap to release the funds
        assert_ok!(Oracle::cancel_swap(
          context.alice.clone(),
          trade_request_mm_id,
        ));
        assert_ok!(Oracle::cancel_swap(context.alice, trade_request_mm2_id,));

        // validate all balance
        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
          INITIAL_20_TDFYS
            .saturating_sub(CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS)
            .saturating_sub(REQUESTER_SWAP_FEE_RATE * CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS)
        );
        assert_eq!(
          Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
          CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
        );
        assert_eq!(
          Adapter::balance_on_hold(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
          Zero::zero()
        );

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &CHARLIE_ACCOUNT_ID),
          // initial balance + swap
          ONE_TDFY + CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS
        );

        let swap_fee = Fees::calculate_swap_fees(
          TEMP_CURRENCY_ID,
          CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
          SwapType::Limit,
          true,
        )
        .fee;

        assert_spendable_balance_is_updated(
          CHARLIE_ACCOUNT_ID,
          TEMP_CURRENCY_ID,
          INITIAL_10000_TEMPS,
          CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
          swap_fee,
        );
        assert_eq!(
          Adapter::balance_on_hold(TEMP_CURRENCY_ID, &CHARLIE_ACCOUNT_ID),
          Zero::zero()
        );

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &DAVE_ACCOUNT_ID),
          ONE_TDFY
        );
        assert_eq!(
          Adapter::balance(TEMP_CURRENCY_ID, &DAVE_ACCOUNT_ID),
          INITIAL_10000_TEMPS
        );
        assert_eq!(
          Adapter::balance_on_hold(TEMP_CURRENCY_ID, &DAVE_ACCOUNT_ID),
          Zero::zero()
        );
      });
    }

    #[test]
    fn market_trade_created_by_market_maker() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![BOB_ACCOUNT_ID, CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        Fees::start_era();
        assert!(Fees::current_era().is_some());
        let current_era = Fees::current_era().unwrap().index;

        let tdfy_amount = 10 * ONE_TDFY;
        let temp_amount = 200 * ONE_TEMP;

        let trade_request_id = context.create_temp_to_tdfy_market_swap_request(
          CHARLIE_ACCOUNT_ID,
          temp_amount,
          tdfy_amount,
          EXTRINSIC_HASH_0,
          SLIPPAGE_5_PERCENTS,
        );

        assert_eq!(
          trade_request_id,
          Hash::from_str("0x73e1c320c34f6de5adadaef0169fbfeaab43d277a95bf02593afc0cbeb496c61")
            .unwrap_or_default()
        );

        let trade_request_mm_id = context.create_tdfy_to_temp_limit_swap_request(
          BOB_ACCOUNT_ID,
          tdfy_amount,
          temp_amount,
          EXTRINSIC_HASH_1,
          SLIPPAGE_2_PERCENTS,
        );

        assert_eq!(
          trade_request_mm_id,
          Hash::from_str("0xe0424aac19ef997f1b76ac20d400aecc2ee0258d9eacb7013c3fcfa2e55bdc67")
            .unwrap_or_default()
        );

        assert_ok!(Oracle::confirm_swap(
          context.alice,
          trade_request_id,
          vec![SwapConfirmation {
            request_id: trade_request_mm_id,
            amount_to_receive: temp_amount,
            amount_to_send: tdfy_amount,
          },],
        ));

        // swap confirmation for charlie (mm user)
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_id,
          status: SwapStatus::Completed,
          account_id: CHARLIE_ACCOUNT_ID,
          currency_from: TEMP_CURRENCY_ID,
          currency_amount_from: temp_amount,
          currency_to: CurrencyId::Tdfy,
          currency_amount_to: tdfy_amount,
          initial_extrinsic_hash: EXTRINSIC_HASH_0,
        }));

        // swap confirmation for bob (mm1)
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_mm_id,
          status: SwapStatus::Completed,
          account_id: BOB_ACCOUNT_ID,
          currency_from: CurrencyId::Tdfy,
          currency_amount_from: tdfy_amount,
          currency_to: TEMP_CURRENCY_ID,
          currency_amount_to: temp_amount,
          initial_extrinsic_hash: EXTRINSIC_HASH_1,
        }));

        // BOB and Charlie swaps are deleted
        assert!(Oracle::swaps(trade_request_id).is_none());
        assert!(Oracle::swaps(trade_request_mm_id).is_none());
        account_swap_is_deleted(CHARLIE_ACCOUNT_ID, trade_request_id);
        account_swap_is_deleted(BOB_ACCOUNT_ID, trade_request_mm_id);

        // make sure all balances match
        assert_eq!(
          Adapter::balance(TEMP_CURRENCY_ID, &context.fees_account_id),
          MARKET_MAKER_SWAP_FEE_RATE * temp_amount
        );
        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &context.fees_account_id),
          // we burned 1 tdfy on start so it should contain 1.2 tdfy now
          ONE_TDFY + MARKET_MAKER_SWAP_LIMIT_FEE_RATE * tdfy_amount
        );

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
          INITIAL_20_TDFYS
            .saturating_sub(tdfy_amount)
            .saturating_sub(MARKET_MAKER_SWAP_LIMIT_FEE_RATE * tdfy_amount)
        );
        assert_eq!(
          Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
          BOB_BUYS_200_TEMPS
        );

        // make sure fees are registered on chain
        let bob_fee = Fees::account_fees(current_era, BOB_ACCOUNT_ID);
        assert_eq!(
          bob_fee.first().unwrap().1.fee,
          MARKET_MAKER_SWAP_LIMIT_FEE_RATE * tdfy_amount
        );
        assert_eq!(bob_fee.first().unwrap().1.amount, tdfy_amount);

        let charlie_fee = Fees::account_fees(current_era, CHARLIE_ACCOUNT_ID);
        assert_eq!(
          charlie_fee.first().unwrap().1.fee,
          MARKET_MAKER_SWAP_FEE_RATE * temp_amount
        );
        assert_eq!(charlie_fee.first().unwrap().1.amount, temp_amount);
      });
    }

    #[test]
    fn limit_trade_created_by_market_maker() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![BOB_ACCOUNT_ID, CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        Fees::start_era();
        assert!(Fees::current_era().is_some());
        let current_era = Fees::current_era().unwrap().index;

        let tdfy_amount = 10 * ONE_TDFY;
        let temp_amount = 200 * ONE_TEMP;

        let trade_request_id = context.create_temp_to_tdfy_limit_swap_request(
          CHARLIE_ACCOUNT_ID,
          temp_amount,
          tdfy_amount,
          EXTRINSIC_HASH_0,
          SLIPPAGE_5_PERCENTS,
        );

        assert_eq!(
          trade_request_id,
          Hash::from_str("0x73e1c320c34f6de5adadaef0169fbfeaab43d277a95bf02593afc0cbeb496c61")
            .unwrap_or_default()
        );

        let trade_request_mm_id = context.create_tdfy_to_temp_limit_swap_request(
          BOB_ACCOUNT_ID,
          tdfy_amount,
          temp_amount,
          EXTRINSIC_HASH_1,
          SLIPPAGE_2_PERCENTS,
        );

        assert_eq!(
          trade_request_mm_id,
          Hash::from_str("0xe0424aac19ef997f1b76ac20d400aecc2ee0258d9eacb7013c3fcfa2e55bdc67")
            .unwrap_or_default()
        );

        assert_ok!(Oracle::confirm_swap(
          context.alice,
          trade_request_id,
          vec![SwapConfirmation {
            request_id: trade_request_mm_id,
            amount_to_receive: temp_amount,
            amount_to_send: tdfy_amount,
          },],
        ));

        // swap confirmation for charlie (mm user)
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_id,
          status: SwapStatus::Completed,
          account_id: CHARLIE_ACCOUNT_ID,
          currency_from: TEMP_CURRENCY_ID,
          currency_amount_from: temp_amount,
          currency_to: CurrencyId::Tdfy,
          currency_amount_to: tdfy_amount,
          initial_extrinsic_hash: EXTRINSIC_HASH_0,
        }));

        // swap confirmation for bob (mm1)
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_mm_id,
          status: SwapStatus::Completed,
          account_id: BOB_ACCOUNT_ID,
          currency_from: CurrencyId::Tdfy,
          currency_amount_from: tdfy_amount,
          currency_to: TEMP_CURRENCY_ID,
          currency_amount_to: temp_amount,
          initial_extrinsic_hash: EXTRINSIC_HASH_1,
        }));

        // BOB and Charlie swaps are deleted
        assert!(Oracle::swaps(trade_request_id).is_none());
        assert!(Oracle::swaps(trade_request_mm_id).is_none());
        account_swap_is_deleted(CHARLIE_ACCOUNT_ID, trade_request_id);
        account_swap_is_deleted(BOB_ACCOUNT_ID, trade_request_mm_id);

        // make sure all balances match
        assert_eq!(
          Adapter::balance(TEMP_CURRENCY_ID, &context.fees_account_id),
          MARKET_MAKER_SWAP_FEE_RATE * temp_amount
        );
        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &context.fees_account_id),
          // we burned 1 tdfy on start so it should contain 1.2 tdfy now
          ONE_TDFY + MARKET_MAKER_SWAP_LIMIT_FEE_RATE * tdfy_amount
        );

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
          INITIAL_20_TDFYS
            .saturating_sub(tdfy_amount)
            .saturating_sub(MARKET_MAKER_SWAP_LIMIT_FEE_RATE * tdfy_amount)
        );
        assert_eq!(
          Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
          BOB_BUYS_200_TEMPS
        );

        // make sure fees are registered on chain
        let bob_fee = Fees::account_fees(current_era, BOB_ACCOUNT_ID);
        assert_eq!(
          bob_fee.first().unwrap().1.fee,
          MARKET_MAKER_SWAP_LIMIT_FEE_RATE * tdfy_amount
        );
        assert_eq!(bob_fee.first().unwrap().1.amount, tdfy_amount);

        let charlie_fee = Fees::account_fees(current_era, CHARLIE_ACCOUNT_ID);
        assert_eq!(
          charlie_fee.first().unwrap().1.fee,
          MARKET_MAKER_SWAP_FEE_RATE * temp_amount
        );
        assert_eq!(charlie_fee.first().unwrap().1.amount, temp_amount);
      });
    }

    #[test]
    fn offer_partially_filled_with_price_equals_to_buying_price_upper_bound() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let fees_account_initial_tdfy_balance =
          Adapter::balance(CurrencyId::Tdfy, &Fees::account_id());

        let trade_request_id = context.create_tdfy_to_temp_market_swap_request(
          BOB_ACCOUNT_ID,
          BOB_SELLS_10_TDFYS,
          BOB_BUYS_200_TEMPS,
          EXTRINSIC_HASH_0,
          SLIPPAGE_5_PERCENTS, // Selling price lower bound is 19 TEMPS/TDFY
        );

        let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
          CHARLIE_ACCOUNT_ID,
          CHARLIE_SELLS_4000_TEMPS,
          CHARLIE_BUYS_200_TDFYS,
          EXTRINSIC_HASH_1,
          SLIPPAGE_4_PERCENTS, // Buying price upper bound is 20.8 TEMPS/TDFY
        );
        let charlie_initial_temp_reserved_balance =
          get_account_reserved(CHARLIE_ACCOUNT_ID, TEMP_CURRENCY_ID);

        // The offer price is set to 20.8 TEMPS/TDFY
        let tdfys_to_confirm_swap = BOB_SELLS_10_TDFYS.saturating_div(2);
        let temps_to_confirm_swap = BOB_BUYS_200_TEMPS
          .saturating_div(2)
          .saturating_add(SLIPPAGE_4_PERCENTS * BOB_BUYS_200_TEMPS.saturating_div(2));
        let trader_swap_fee = Fees::calculate_swap_fees(
          CurrencyId::Tdfy,
          tdfys_to_confirm_swap,
          SwapType::Market,
          false,
        )
        .fee;
        let market_maker_swap_fee = Fees::calculate_swap_fees(
          TEMP_CURRENCY_ID,
          temps_to_confirm_swap,
          SwapType::Limit,
          true,
        )
        .fee;

        assert_ok!(Oracle::confirm_swap(
          context.alice,
          trade_request_id,
          vec![SwapConfirmation {
            request_id: trade_request_mm_id,
            // 5 TDFYs to 104 TEMPs, so offer price is 20.8 TEMPS/TDFY
            amount_to_receive: tdfys_to_confirm_swap,
            amount_to_send: temps_to_confirm_swap,
          }],
        ));

        // Events are emitted
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_id,
          status: SwapStatus::PartiallyFilled,
          account_id: BOB_ACCOUNT_ID,
          currency_from: CurrencyId::Tdfy,
          currency_amount_from: tdfys_to_confirm_swap,
          currency_to: TEMP_CURRENCY_ID,
          currency_amount_to: temps_to_confirm_swap,
          initial_extrinsic_hash: EXTRINSIC_HASH_0,
        }));

        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_mm_id,
          status: SwapStatus::PartiallyFilled,
          account_id: CHARLIE_ACCOUNT_ID,
          currency_from: TEMP_CURRENCY_ID,
          currency_amount_from: temps_to_confirm_swap,
          currency_to: CurrencyId::Tdfy,
          currency_amount_to: tdfys_to_confirm_swap,
          initial_extrinsic_hash: EXTRINSIC_HASH_1,
        }));

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
          INITIAL_20_TDFYS - tdfys_to_confirm_swap - trader_swap_fee
        );
        assert_eq!(
          Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
          temps_to_confirm_swap
        );

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &CHARLIE_ACCOUNT_ID),
          INITIAL_20_TDFYS + tdfys_to_confirm_swap
        );
        assert_eq!(
          get_account_reserved(CHARLIE_ACCOUNT_ID, TEMP_CURRENCY_ID),
          charlie_initial_temp_reserved_balance - temps_to_confirm_swap - market_maker_swap_fee
        );

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &Fees::account_id()),
          fees_account_initial_tdfy_balance + trader_swap_fee // Fees account has 1 TDFY, which is an existential deposit initially
        );
        assert_eq!(
          Adapter::balance(TEMP_CURRENCY_ID, &Fees::account_id()),
          market_maker_swap_fee
        );

        // Trader market swap is removed from Swaps
        assert!(Oracle::swaps(trade_request_id).is_none());

        // Trader market swap is removed from AccountSwaps
        account_swap_is_deleted(BOB_ACCOUNT_ID, trade_request_id);

        // Charlie's limit swap in both Swaps and AccountSwaps and kept and their status are updated from Pending to PartiallyFilled
        swap_exists_with_status(trade_request_mm_id, SwapStatus::PartiallyFilled);
        account_swap_exists_with_status(
          CHARLIE_ACCOUNT_ID,
          trade_request_mm_id,
          SwapStatus::PartiallyFilled,
        );

        // Trader's reserved funds is fully released as its swap type is market swap
        assert_eq!(get_account_reserved(BOB_ACCOUNT_ID, CurrencyId::Tdfy), 0);

        // Market Maker's reserved funds is reduced by confirmed swap sold amount and fee
        assert_eq!(
          get_account_reserved(CHARLIE_ACCOUNT_ID, TEMP_CURRENCY_ID),
          charlie_initial_temp_reserved_balance - temps_to_confirm_swap - market_maker_swap_fee
        );
      });
    }

    #[test]
    fn offer_full_filled_with_price_equals_to_buying_price_upper_bound() {
      const INITIAL_200_TDFYS: Balance = 200 * ONE_TDFY;
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_200_TDFYS)
          .mint_temp2(CHARLIE_ACCOUNT_ID, 1_000 * ONE_TEMP2);

        let fees_account_initial_tdfy_balance =
          Adapter::balance(CurrencyId::Tdfy, &Fees::account_id());

        let tdfy_amount = 100 * ONE_TDFY;
        let temp2_amount = ONE_TEMP2;

        let trade_request_id = add_new_swap_and_assert_results(
          BOB_ACCOUNT_ID,
          CurrencyId::Tdfy,
          tdfy_amount,
          TEMP2_CURRENCY_ID,
          temp2_amount,
          CURRENT_BLOCK_NUMBER,
          EXTRINSIC_HASH_0,
          MarketMakers::<Test>::get(&BOB_ACCOUNT_ID).is_some(),
          SwapType::Market,
          SLIPPAGE_5_PERCENTS,
        );

        let trade_request_mm_id = add_new_swap_and_assert_results(
          CHARLIE_ACCOUNT_ID,
          TEMP2_CURRENCY_ID,
          temp2_amount,
          CurrencyId::Tdfy,
          tdfy_amount,
          CURRENT_BLOCK_NUMBER,
          EXTRINSIC_HASH_1,
          MarketMakers::<Test>::get(&CHARLIE_ACCOUNT_ID).is_some(),
          SwapType::Limit,
          SLIPPAGE_4_PERCENTS,
        );
        let charlie_initial_temp_reserved_balance =
          get_account_reserved(CHARLIE_ACCOUNT_ID, TEMP2_CURRENCY_ID);

        // The offer price is set to 0.0104 TEMP2/TDFY
        let temp2_to_confirm_swap = temp2_amount.saturating_add(SLIPPAGE_4_PERCENTS * temp2_amount);

        let trader_swap_fee =
          Fees::calculate_swap_fees(CurrencyId::Tdfy, tdfy_amount, SwapType::Market, false).fee;
        let market_maker_swap_fee = Fees::calculate_swap_fees(
          TEMP2_CURRENCY_ID,
          temp2_to_confirm_swap,
          SwapType::Limit,
          true,
        )
        .fee;

        assert_ok!(Oracle::confirm_swap(
          context.alice,
          trade_request_id,
          vec![SwapConfirmation {
            request_id: trade_request_mm_id,
            amount_to_receive: tdfy_amount,
            amount_to_send: temp2_to_confirm_swap,
          }],
        ));

        // Events are emitted
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_id,
          status: SwapStatus::Completed,
          account_id: BOB_ACCOUNT_ID,
          currency_from: CurrencyId::Tdfy,
          currency_amount_from: tdfy_amount,
          currency_to: TEMP2_CURRENCY_ID,
          currency_amount_to: temp2_to_confirm_swap,
          initial_extrinsic_hash: EXTRINSIC_HASH_0,
        }));

        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_mm_id,
          status: SwapStatus::Completed,
          account_id: CHARLIE_ACCOUNT_ID,
          currency_from: TEMP2_CURRENCY_ID,
          currency_amount_from: temp2_to_confirm_swap,
          currency_to: CurrencyId::Tdfy,
          currency_amount_to: tdfy_amount,
          initial_extrinsic_hash: EXTRINSIC_HASH_1,
        }));

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
          INITIAL_200_TDFYS - tdfy_amount - trader_swap_fee
        );
        assert_eq!(
          Adapter::balance(TEMP2_CURRENCY_ID, &BOB_ACCOUNT_ID),
          temp2_to_confirm_swap
        );

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &CHARLIE_ACCOUNT_ID),
          INITIAL_20_TDFYS + tdfy_amount
        );
        assert_eq!(
          get_account_reserved(CHARLIE_ACCOUNT_ID, TEMP2_CURRENCY_ID),
          charlie_initial_temp_reserved_balance - temp2_to_confirm_swap - market_maker_swap_fee
        );

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &Fees::account_id()),
          fees_account_initial_tdfy_balance + trader_swap_fee // Fees account has 1 TDFY, which is an existential deposit initially
        );
        assert_eq!(
          Adapter::balance(TEMP2_CURRENCY_ID, &Fees::account_id()),
          market_maker_swap_fee
        );

        // Trader market swap is removed from Swaps
        assert!(Oracle::swaps(trade_request_id).is_none());

        // Trader market swap is removed from AccountSwaps
        account_swap_is_deleted(BOB_ACCOUNT_ID, trade_request_id);

        // Market maker's limit swap is removed from Swaps
        assert!(Oracle::swaps(trade_request_mm_id).is_none());

        // Trader market swap is removed from AccountSwaps
        account_swap_is_deleted(BOB_ACCOUNT_ID, trade_request_id);

        // Trader's reserved funds is fully released as its swap type is market swap
        assert_eq!(get_account_reserved(BOB_ACCOUNT_ID, CurrencyId::Tdfy), 0);

        // Market Maker's reserved funds is reduced by confirmed swap sold amount and fee
        assert_eq!(
          get_account_reserved(CHARLIE_ACCOUNT_ID, TEMP2_CURRENCY_ID),
          charlie_initial_temp_reserved_balance - temp2_to_confirm_swap - market_maker_swap_fee
        );
      });
    }

    #[test]
    fn offer_partially_filled_with_price_equals_to_selling_price_lower_bound() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let fees_account_initial_tdfy_balance =
          Adapter::balance(CurrencyId::Tdfy, &Fees::account_id());

        let trade_request_id = context.create_tdfy_to_temp_market_swap_request(
          BOB_ACCOUNT_ID,
          BOB_SELLS_10_TDFYS,
          BOB_BUYS_200_TEMPS,
          EXTRINSIC_HASH_0,
          SLIPPAGE_5_PERCENTS, // Selling price lower bound is 19 TEMPS/TDFY
        );

        let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
          CHARLIE_ACCOUNT_ID,
          CHARLIE_SELLS_4000_TEMPS,
          CHARLIE_BUYS_200_TDFYS,
          EXTRINSIC_HASH_1,
          SLIPPAGE_4_PERCENTS, // Buying price upper bound is 20.8 TEMPS/TDFY
        );
        let charlie_initial_temp_reserved_balance =
          get_account_reserved(CHARLIE_ACCOUNT_ID, TEMP_CURRENCY_ID);

        // The offer price is set to 19 TEMPS/TDFY
        let tdfys_to_confirm_swap = BOB_SELLS_10_TDFYS.saturating_div(2);
        let temps_to_confirm_swap = BOB_BUYS_200_TEMPS
          .saturating_div(2)
          .saturating_sub(SLIPPAGE_5_PERCENTS * BOB_BUYS_200_TEMPS.saturating_div(2));
        let trader_swap_fee = Fees::calculate_swap_fees(
          CurrencyId::Tdfy,
          tdfys_to_confirm_swap,
          SwapType::Market,
          false,
        )
        .fee;
        let market_maker_swap_fee = Fees::calculate_swap_fees(
          TEMP_CURRENCY_ID,
          temps_to_confirm_swap,
          SwapType::Limit,
          true,
        )
        .fee;

        assert_ok!(Oracle::confirm_swap(
          context.alice,
          trade_request_id,
          vec![SwapConfirmation {
            request_id: trade_request_mm_id,
            // 5 TDFYs to 104 TEMPs, so offer price is 20.8 TEMPS/TDFY
            amount_to_receive: tdfys_to_confirm_swap,
            amount_to_send: temps_to_confirm_swap,
          }],
        ));

        // Events are emitted
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_id,
          status: SwapStatus::PartiallyFilled,
          account_id: BOB_ACCOUNT_ID,
          currency_from: CurrencyId::Tdfy,
          currency_amount_from: tdfys_to_confirm_swap,
          currency_to: TEMP_CURRENCY_ID,
          currency_amount_to: temps_to_confirm_swap,
          initial_extrinsic_hash: EXTRINSIC_HASH_0,
        }));

        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_mm_id,
          status: SwapStatus::PartiallyFilled,
          account_id: CHARLIE_ACCOUNT_ID,
          currency_from: TEMP_CURRENCY_ID,
          currency_amount_from: temps_to_confirm_swap,
          currency_to: CurrencyId::Tdfy,
          currency_amount_to: tdfys_to_confirm_swap,
          initial_extrinsic_hash: EXTRINSIC_HASH_1,
        }));

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
          INITIAL_20_TDFYS - tdfys_to_confirm_swap - trader_swap_fee
        );
        assert_eq!(
          Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
          temps_to_confirm_swap
        );

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &CHARLIE_ACCOUNT_ID),
          INITIAL_20_TDFYS + tdfys_to_confirm_swap
        );
        assert_eq!(
          get_account_reserved(CHARLIE_ACCOUNT_ID, TEMP_CURRENCY_ID),
          charlie_initial_temp_reserved_balance - temps_to_confirm_swap - market_maker_swap_fee
        );

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &Fees::account_id()),
          fees_account_initial_tdfy_balance + trader_swap_fee // Fees account has 1 TDFY, which is an existential deposit initially
        );
        assert_eq!(
          Adapter::balance(TEMP_CURRENCY_ID, &Fees::account_id()),
          market_maker_swap_fee
        );

        // Trader market swap is removed from Swaps
        assert!(Oracle::swaps(trade_request_id).is_none());

        // Trader market swap is removed from AccountSwaps
        account_swap_is_deleted(BOB_ACCOUNT_ID, trade_request_id);

        // Charlie's limit swap in both Swaps and AccountSwaps and kept and their status are updated from Pending to PartiallyFilled
        swap_exists_with_status(trade_request_mm_id, SwapStatus::PartiallyFilled);
        account_swap_exists_with_status(
          CHARLIE_ACCOUNT_ID,
          trade_request_mm_id,
          SwapStatus::PartiallyFilled,
        );

        // Trader's reserved funds is fully released as its swap type is market swap
        assert_eq!(get_account_reserved(BOB_ACCOUNT_ID, CurrencyId::Tdfy), 0);

        // Market Maker's reserved funds is reduced by confirmed swap sold amount and fee
        assert_eq!(
          get_account_reserved(CHARLIE_ACCOUNT_ID, TEMP_CURRENCY_ID),
          charlie_initial_temp_reserved_balance - temps_to_confirm_swap - market_maker_swap_fee
        );
      });
    }

    #[test]
    fn offer_full_filled_with_price_equals_to_selling_price_lower_bound() {
      const INITIAL_200_TDFYS: Balance = 200 * ONE_TDFY;
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_200_TDFYS)
          .mint_temp2(CHARLIE_ACCOUNT_ID, 1_000 * ONE_TEMP2);

        let fees_account_initial_tdfy_balance =
          Adapter::balance(CurrencyId::Tdfy, &Fees::account_id());

        // Create swaps between 100 TDFY and 1 TEMP2
        let tdfy_amount = 100 * ONE_TDFY;
        let temp2_amount = ONE_TEMP2;

        let trade_request_id = add_new_swap_and_assert_results(
          BOB_ACCOUNT_ID,
          CurrencyId::Tdfy,
          tdfy_amount,
          TEMP2_CURRENCY_ID,
          temp2_amount,
          CURRENT_BLOCK_NUMBER,
          EXTRINSIC_HASH_0,
          MarketMakers::<Test>::get(&BOB_ACCOUNT_ID).is_some(),
          SwapType::Market,
          SLIPPAGE_5_PERCENTS,
        );
        let trade_request_mm_id = add_new_swap_and_assert_results(
          CHARLIE_ACCOUNT_ID,
          TEMP2_CURRENCY_ID,
          temp2_amount,
          CurrencyId::Tdfy,
          tdfy_amount,
          CURRENT_BLOCK_NUMBER,
          EXTRINSIC_HASH_1,
          MarketMakers::<Test>::get(&CHARLIE_ACCOUNT_ID).is_some(),
          SwapType::Limit,
          SLIPPAGE_4_PERCENTS,
        );

        // The offer price is set to 0.0096 TEMP2/TDFY
        let temp2_to_confirm_swap = temp2_amount.saturating_sub(SLIPPAGE_4_PERCENTS * temp2_amount);

        let trader_swap_fee =
          Fees::calculate_swap_fees(CurrencyId::Tdfy, tdfy_amount, SwapType::Market, false).fee;
        let market_maker_swap_fee = Fees::calculate_swap_fees(
          TEMP2_CURRENCY_ID,
          temp2_to_confirm_swap,
          SwapType::Limit,
          true,
        )
        .fee;

        assert_ok!(Oracle::confirm_swap(
          context.alice,
          trade_request_id,
          vec![SwapConfirmation {
            request_id: trade_request_mm_id,
            amount_to_receive: tdfy_amount,
            amount_to_send: temp2_to_confirm_swap,
          }],
        ));

        // Events are emitted
        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_id,
          status: SwapStatus::Completed,
          account_id: BOB_ACCOUNT_ID,
          currency_from: CurrencyId::Tdfy,
          currency_amount_from: tdfy_amount,
          currency_to: TEMP2_CURRENCY_ID,
          currency_amount_to: temp2_to_confirm_swap,
          initial_extrinsic_hash: EXTRINSIC_HASH_0,
        }));

        System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
          request_id: trade_request_mm_id,
          status: SwapStatus::Completed,
          account_id: CHARLIE_ACCOUNT_ID,
          currency_from: TEMP2_CURRENCY_ID,
          currency_amount_from: temp2_to_confirm_swap,
          currency_to: CurrencyId::Tdfy,
          currency_amount_to: tdfy_amount,
          initial_extrinsic_hash: EXTRINSIC_HASH_1,
        }));

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
          INITIAL_200_TDFYS - tdfy_amount - trader_swap_fee
        );
        assert_eq!(
          Adapter::balance(TEMP2_CURRENCY_ID, &BOB_ACCOUNT_ID),
          temp2_to_confirm_swap
        );

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &CHARLIE_ACCOUNT_ID),
          INITIAL_20_TDFYS + tdfy_amount
        );
        assert_eq!(
          get_account_reserved(CHARLIE_ACCOUNT_ID, TEMP2_CURRENCY_ID),
          0
        );

        assert_eq!(
          Adapter::balance(CurrencyId::Tdfy, &Fees::account_id()),
          fees_account_initial_tdfy_balance + trader_swap_fee // Fees account has 1 TDFY, which is an existential deposit initially
        );
        assert_eq!(
          Adapter::balance(TEMP2_CURRENCY_ID, &Fees::account_id()),
          market_maker_swap_fee
        );

        // Trader market swap is removed from Swaps
        assert!(Oracle::swaps(trade_request_id).is_none());

        // Trader market swap is removed from AccountSwaps
        account_swap_is_deleted(BOB_ACCOUNT_ID, trade_request_id);

        // Market maker's limit swap is removed from Swaps
        assert!(Oracle::swaps(trade_request_mm_id).is_none());

        // Trader market swap is removed from AccountSwaps
        account_swap_is_deleted(BOB_ACCOUNT_ID, trade_request_id);

        // Trader's reserved funds is fully released as its swap type is market swap
        assert_eq!(get_account_reserved(BOB_ACCOUNT_ID, CurrencyId::Tdfy), 0);

        // Market Maker's reserved funds is fully released as it's completed
        assert_eq!(
          get_account_reserved(CHARLIE_ACCOUNT_ID, TEMP2_CURRENCY_ID),
          0
        );
      });
    }

    #[test]
    fn swap_between_the_same_account() {
      new_test_ext().execute_with(|| {
        const BOB_INITIAL_10000_TEMPS: Balance = 10_000 * ONE_TEMP;

        let context = Context::default()
          .set_oracle_status(true)
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(BOB_ACCOUNT_ID, BOB_INITIAL_10000_TEMPS);

        let trade_request_id = context.create_tdfy_to_temp_limit_swap_request(
          BOB_ACCOUNT_ID,
          BOB_SELLS_10_TDFYS,
          BOB_BUYS_200_TEMPS,
          EXTRINSIC_HASH_0,
          SLIPPAGE_2_PERCENTS,
        );

        assert_eq!(
          trade_request_id,
          Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
            .unwrap_or_default()
        );

        const BOB_SELLS_400_TEMPS: Balance = 400 * ONE_TEMP;
        const BOB_BUYS_10_TDFYS: Balance = 10 * ONE_TDFY;
        let context = Context::default().set_market_makers(vec![BOB_ACCOUNT_ID]);
        let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
          BOB_ACCOUNT_ID,
          BOB_SELLS_400_TEMPS,
          BOB_BUYS_10_TDFYS,
          EXTRINSIC_HASH_0,
          SLIPPAGE_5_PERCENTS,
        );

        assert_eq!(
          trade_request_mm_id,
          Hash::from_str("0xe0424aac19ef997f1b76ac20d400aecc2ee0258d9eacb7013c3fcfa2e55bdc67")
            .unwrap_or_default()
        );

        // partial filling
        const BOB_FILLING_BUYS_10_TDFYS: Balance = 10 * ONE_TDFY;
        const BOB_FILLING_SELLS_400_TDFYS: Balance = 400 * ONE_TEMP;
        assert_ok!(Oracle::confirm_swap(
          context.alice.clone(),
          trade_request_id,
          vec![SwapConfirmation {
            request_id: trade_request_mm_id,
            amount_to_receive: BOB_FILLING_BUYS_10_TDFYS,
            amount_to_send: BOB_FILLING_SELLS_400_TDFYS,
          },],
        ));

        // BOB: make sure the CLIENT current trade is partially filled and correctly updated
        assert!(Oracle::swaps(trade_request_id).is_none());
        assert!(Oracle::swaps(trade_request_mm_id).is_none());

        // cant send another trade confirmation as the request should be deleted
        // we do expect `InvalidRequestId`
        assert_noop!(
          Oracle::confirm_swap(context.alice, trade_request_id, vec![],),
          Error::<Test>::InvalidRequestId
        );

        // validate all balance
        assert_eq!(
          Adapter::reducible_balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID, false),
          // we should refund the extra fees paid on the slippage value
          INITIAL_20_TDFYS.saturating_sub(REQUESTER_SWAP_FEE_RATE * BOB_SELLS_10_TDFYS)
        );

        assert_eq!(
          Adapter::reducible_balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID, false),
          BOB_INITIAL_10000_TEMPS.saturating_sub(MARKET_MAKER_SWAP_FEE_RATE * BOB_SELLS_400_TEMPS)
        );

        assert_eq!(
          Adapter::balance_on_hold(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
          Zero::zero()
        );
        assert_eq!(
          Adapter::balance_on_hold(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
          Zero::zero()
        );
      });
    }

    #[test]
    fn trade_from_zemp_to_temp() {
      new_test_ext().execute_with(|| {
        const BOB_INITIAL_ZEMPS: Balance = 10 * ONE_ZEMP;
        const CHARLIE_INITIAL_TEMPS: Balance = 2 * ONE_TEMP;

        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_zemp(BOB_ACCOUNT_ID, BOB_INITIAL_ZEMPS)
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_TEMPS);

        const ZEMPS_AMOUNT: Balance = BOB_INITIAL_ZEMPS.saturating_div(3);
        const TEMPS_AMOUNT: Balance = CHARLIE_INITIAL_TEMPS.saturating_div(3);

        let trade_request_id = context.create_zemp_to_temp_market_swap_request(
          BOB_ACCOUNT_ID,
          ZEMPS_AMOUNT,
          TEMPS_AMOUNT,
          EXTRINSIC_HASH_0,
          SLIPPAGE_2_PERCENTS,
        );

        assert_eq!(
          trade_request_id,
          Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
            .unwrap_or_default()
        );

        let trade_request_mm_id = context.create_temp_to_zemp_limit_swap_request(
          CHARLIE_ACCOUNT_ID,
          TEMPS_AMOUNT,
          ZEMPS_AMOUNT,
          EXTRINSIC_HASH_1,
          SLIPPAGE_0_PERCENT,
        );

        assert_eq!(
          trade_request_mm_id,
          Hash::from_str("0x9ee76e89d3eae9ddad2e0b731e29ddcfa0781f7035600c5eb885637592e1d2c2")
            .unwrap_or_default()
        );

        // partial filling
        assert_ok!(Oracle::confirm_swap(
          context.alice,
          trade_request_id,
          vec![
            // charlie
            SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: ZEMPS_AMOUNT,
              amount_to_send: TEMPS_AMOUNT,
            },
          ],
        ));
      });
    }

    #[test]
    fn trade_from_temp_to_zemp() {
      new_test_ext().execute_with(|| {
        const BOB_INITIAL_TEMPS: Balance = 900_000 * ONE_TEMP;
        const CHARLIE_INITIAL_ZEMPS: Balance = 900_000 * ONE_ZEMP;

        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_temp(BOB_ACCOUNT_ID, BOB_INITIAL_TEMPS)
          .mint_zemp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_ZEMPS);

        const BOB_SELLS_TEMPS: Balance = 10_000_000;
        const BOB_BUYS_ZEMPS: Balance = 12_500_000_000_000_000_000;

        let trade_request_id = context.create_temp_to_zemp_market_swap_request(
          BOB_ACCOUNT_ID,
          BOB_SELLS_TEMPS,
          BOB_BUYS_ZEMPS,
          EXTRINSIC_HASH_0,
          SLIPPAGE_2_PERCENTS,
        );

        assert_eq!(
          trade_request_id,
          Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
            .unwrap_or_default()
        );

        // 0.80478930
        const CHARLIE_SELLS_ZEMPS: Balance = 1_000_000_000_000_000_000_000;
        // 9.838500000000000000
        const CHARLIE_BUYS_TEMPS: Balance = 800_000_000;

        let trade_request_mm_id = context.create_zemp_to_temp_limit_swap_request(
          CHARLIE_ACCOUNT_ID,
          CHARLIE_SELLS_ZEMPS,
          CHARLIE_BUYS_TEMPS,
          EXTRINSIC_HASH_1,
          SLIPPAGE_0_PERCENT,
        );

        assert_eq!(
          trade_request_mm_id,
          Hash::from_str("0x9ee76e89d3eae9ddad2e0b731e29ddcfa0781f7035600c5eb885637592e1d2c2")
            .unwrap_or_default()
        );

        // partial filling
        assert_ok!(Oracle::confirm_swap(
          context.alice,
          trade_request_id,
          vec![
            // charlie
            SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: BOB_SELLS_TEMPS,
              amount_to_send: BOB_BUYS_ZEMPS,
            },
          ],
        ));
      });
    }
  }

  mod fails_when {
    use super::*;

    #[test]
    fn oracle_is_paused() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(false)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice,
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::OraclePaused
        );
      });
    }

    #[test]
    fn not_signed() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            RuntimeOrigin::none(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          BadOrigin
        );
      });
    }

    #[test]
    fn not_signed_by_sender() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            context.bob,
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::AccessDenied
        );
      });
    }

    #[test]
    fn request_id_is_invalid() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        const INVALID_REQUEST_ID: H256 = H256::zero();
        assert_noop!(
          Oracle::confirm_swap(
            context.alice,
            INVALID_REQUEST_ID,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::InvalidRequestId
        );
      });
    }

    #[test]
    fn trade_request_status_is_invalid() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        for invalid_status in vec![
          SwapStatus::Cancelled,
          SwapStatus::Completed,
          SwapStatus::Rejected,
        ] {
          Swaps::<Test>::mutate(trade_request_id, |request| {
            if let Some(trade_request) = request {
              trade_request.status = invalid_status.clone()
            }
          });

          assert_noop!(
            Oracle::confirm_swap(
              context.alice.clone(),
              trade_request_id,
              vec![SwapConfirmation {
                request_id: trade_request_mm_id,
                amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
                amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
              },],
            ),
            Error::<Test>::InvalidSwapStatus
          );
        }
      });
    }

    #[test]
    fn market_maker_request_status_is_invalid() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        for invalid_status in vec![
          SwapStatus::Cancelled,
          SwapStatus::Completed,
          SwapStatus::Rejected,
        ] {
          Swaps::<Test>::mutate(trade_request_mm_id, |request| {
            if let Some(trade_request) = request {
              trade_request.status = invalid_status.clone()
            }
          });

          assert_noop!(
            Oracle::confirm_swap(
              context.alice.clone(),
              trade_request_id,
              vec![SwapConfirmation {
                request_id: trade_request_mm_id,
                amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
                amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
              },],
            ),
            Error::<Test>::InvalidMarketMakerSwapRequestStatus
          );
        }
      });
    }

    #[test]
    fn market_maker_request_id_is_invalid() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );

        const INVALID_REQUEST_ID: H256 = H256::zero();

        assert_noop!(
          Oracle::confirm_swap(
            context.alice,
            trade_request_id,
            vec![SwapConfirmation {
              request_id: INVALID_REQUEST_ID,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::InvalidMarketMakerRequestId { index: 0 }
        );
      });
    }

    #[test]
    fn offer_price_is_greater_than_buying_price_upper_bound() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice,
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: ONE_TDFY,
              amount_to_send: BOB_BUYS_200_TEMPS,
            }],
          ),
          Error::<Test>::OfferIsGreaterThanMarketMakerSwapUpperBound { index: 0 }
        );
      });
    }

    #[test]
    fn offer_price_is_less_than_selling_price_lower_bound() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id = context.create_tdfy_to_temp_limit_swap_request(
          BOB_ACCOUNT_ID,
          BOB_SELLS_10_TDFYS,
          BOB_BUYS_200_TEMPS,
          EXTRINSIC_HASH_0,
          SLIPPAGE_5_PERCENTS, // Swap Lower Bound: 19 TEMPS/TDFY
        );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice,
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: BOB_SELLS_10_TDFYS
                .saturating_add(SLIPPAGE_5_PERCENTS * BOB_SELLS_10_TDFYS)
                .saturating_add(ONE_TDFY),
              amount_to_send: BOB_BUYS_200_TEMPS,
            }], // Offer Price: 17.3 TEMPS/TDFY
          ),
          Error::<Test>::OfferIsLessThanSwapLowerBound { index: 0 }
        );
      });
    }

    #[test]
    fn market_maker_swap_does_not_have_enough_funds() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
          CHARLIE_ACCOUNT_ID,
          BOB_BUYS_200_TEMPS.saturating_div(5),
          BOB_SELLS_10_TDFYS.saturating_div(5),
          EXTRINSIC_HASH_1,
          SLIPPAGE_4_PERCENTS,
        );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice,
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::MarketMakerSwapHasNotEnoughTokenLeftToSell
        );
      });
    }

    #[test]
    fn offer_sells_more_base_asset_than_swap_amount() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![BOB_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id = context.create_temp_to_tdfy_limit_swap_request(
          CHARLIE_ACCOUNT_ID,
          200 * ONE_TEMP,
          10 * ONE_TDFY,
          EXTRINSIC_HASH_0,
          SLIPPAGE_2_PERCENTS,
        );

        let trade_request_mm_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice,
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: BOB_BUYS_200_TEMPS,
              amount_to_send: BOB_SELLS_10_TDFYS.saturating_add(1),
            },],
          ),
          Error::<Test>::RequestCannotOversell
        );
      });
    }

    #[test]
    fn offer_buys_more_base_asset_than_swap_amount() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice,
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: BOB_SELLS_10_TDFYS.saturating_add(1),
              amount_to_send: BOB_BUYS_200_TEMPS,
            },],
          ),
          Error::<Test>::RequestCannotOversell
        );
      });
    }

    #[test]
    fn market_maker_buy_token_is_different_from_swap_sell_token() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS)
          .mint_temp2(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );

        let trade_request_mm_id = add_new_swap_and_assert_results(
          CHARLIE_ACCOUNT_ID,
          TEMP_CURRENCY_ID, // From TEMP
          CHARLIE_SELLS_4000_TEMPS,
          TEMP2_CURRENCY_ID, // To TEMP2
          CHARLIE_BUYS_200_TDFYS,
          CURRENT_BLOCK_NUMBER,
          EXTRINSIC_HASH_1,
          true,
          SwapType::Limit,
          SLIPPAGE_4_PERCENTS,
        );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice,
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::BuySellAssetMismatch
        );
      });
    }

    #[test]
    fn market_maker_sell_token_is_different_from_swap_buy_token() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS)
          .mint_temp2(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );

        let trade_request_mm_id = add_new_swap_and_assert_results(
          CHARLIE_ACCOUNT_ID,
          TEMP2_CURRENCY_ID, // From TEMP2
          CHARLIE_SELLS_4000_TEMPS,
          TEMP_CURRENCY_ID, // To TEMP
          CHARLIE_BUYS_200_TDFYS,
          CURRENT_BLOCK_NUMBER,
          EXTRINSIC_HASH_1,
          true,
          SwapType::Limit,
          SLIPPAGE_2_PERCENTS,
        );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::BuySellAssetMismatch
        );
      });
    }

    #[test]
    fn non_market_maker_added_market_maker_swap() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS)
          .mint_temp2(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );

        let trade_request_mm_id = add_new_swap_and_assert_results(
          CHARLIE_ACCOUNT_ID,
          TEMP_CURRENCY_ID,
          CHARLIE_SELLS_4000_TEMPS,
          CurrencyId::Tdfy,
          CHARLIE_BUYS_200_TDFYS,
          CURRENT_BLOCK_NUMBER,
          EXTRINSIC_HASH_1,
          true,
          SwapType::Limit,
          SLIPPAGE_2_PERCENTS,
        );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::NonMarketMakerSwap
        );
      });
    }

    #[test]
    fn market_maker_swap_type_is_not_limit() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS)
          .mint_temp2(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );

        let trade_request_mm_id = add_new_swap_and_assert_results(
          CHARLIE_ACCOUNT_ID,
          TEMP_CURRENCY_ID,
          CHARLIE_SELLS_4000_TEMPS,
          CurrencyId::Tdfy,
          CHARLIE_BUYS_200_TDFYS,
          CURRENT_BLOCK_NUMBER,
          EXTRINSIC_HASH_1,
          true,
          SwapType::Market,
          SLIPPAGE_2_PERCENTS,
        );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::MarketMakerSwapTypeIsNotLimit
        );
      });
    }

    #[test]
    fn trader_not_hold_enough_fund_to_swap() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        // Release selling amount from Bob's on hold balance
        Adapter::release(CurrencyId::Tdfy, &BOB_ACCOUNT_ID, 10 * ONE_TDFY, false)
          .expect("Should be able to release swap amount from Bob's reserved balance");

        assert_noop!(
          Oracle::confirm_swap(
            context.alice,
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::SellerDoesNotHoldEnoughFundToSellAndPaySwapFees // This error is thrown so TransferTraderSwapAssetsToMarketMakerFailed should never be triggered
        );
      });
    }

    #[test]
    fn trader_not_hold_enough_fund_to_pay_swap_fee() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
          CHARLIE_ACCOUNT_ID,
          200 * ONE_TEMP,
          10 * ONE_TDFY,
          EXTRINSIC_HASH_1,
          SLIPPAGE_2_PERCENTS,
        );

        let amount_and_fee =
          Fees::calculate_swap_fees(CurrencyId::Tdfy, 10 * ONE_TDFY, SwapType::Limit, false);

        // Release fee from Bob's on hold balance
        Adapter::transfer_held(
          CurrencyId::Tdfy,
          &BOB_ACCOUNT_ID,
          &ALICE_ACCOUNT_ID,
          amount_and_fee.fee,
          false,
          false,
        )
        .expect("Should be able to withdraw swap fee from Bob's reserved balance");

        assert_noop!(
          Oracle::confirm_swap(
            context.alice,
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: 10 * ONE_TDFY,
              amount_to_send: 200 * ONE_TEMP,
            },],
          ),
          Error::<Test>::SellerDoesNotHoldEnoughFundToSellAndPaySwapFees // This error is thrown so TransferTraderSwapFeeFailed should never be triggered
        );
      });
    }

    #[test]
    fn market_maker_not_hold_enough_fund_to_swap() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        let quote_asset_amount =
          (4000 * ONE_TEMP).saturating_add(SLIPPAGE_4_PERCENTS * (4000 * ONE_TEMP));
        let quote_asset_fee =
          Fees::calculate_swap_fees(TEMP_CURRENCY_ID, quote_asset_amount, SwapType::Limit, true)
            .fee;

        // Transfer Charlie's on hold selling tokens to Alice
        Adapter::transfer_held(
          TEMP_CURRENCY_ID,
          &CHARLIE_ACCOUNT_ID,
          &ALICE_ACCOUNT_ID,
          quote_asset_amount.saturating_add(quote_asset_fee),
          false,
          false,
        )
        .expect("Should be able to release swap fee from Charlie's reserved balance");

        assert_noop!(
          Oracle::confirm_swap(
            context.alice,
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::SellerDoesNotHoldEnoughFundToSellAndPaySwapFees // This error is thrown so TransferMarketMakerSwapAssetsToTraderFailed should never be triggered
        );
      });
    }

    #[test]
    fn market_maker_not_hold_enough_fund_to_pay_swap_fee() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
          CHARLIE_ACCOUNT_ID,
          200 * ONE_TEMP,
          10 * ONE_TDFY,
          EXTRINSIC_HASH_1,
          SLIPPAGE_2_PERCENTS,
        );

        let quote_asset_amount =
          (200 * ONE_TEMP).saturating_add(SLIPPAGE_2_PERCENTS * (200 * ONE_TEMP));
        let quote_asset_fee =
          Fees::calculate_swap_fees(TEMP_CURRENCY_ID, quote_asset_amount, SwapType::Limit, true)
            .fee;

        // Release fee from Charlie's on hold balance
        Adapter::transfer_held(
          TEMP_CURRENCY_ID,
          &CHARLIE_ACCOUNT_ID,
          &ALICE_ACCOUNT_ID,
          quote_asset_amount.saturating_add(quote_asset_fee),
          false,
          false,
        )
        .expect("Should be able to withdraw swap fee from Charlie's reserved balance");

        assert_noop!(
          Oracle::confirm_swap(
            context.alice,
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: 10 * ONE_TDFY,
              amount_to_send: 200 * ONE_TEMP,
            },],
          ),
          Error::<Test>::SellerDoesNotHoldEnoughFundToSellAndPaySwapFees // This error is thrown so TransferMarketMakerSwapFeeFailed should never be triggered
        );
      });
    }
  }
}

mod add_market_pair {
  use super::*;

  #[test]
  fn succeed() {
    new_test_ext().execute_with(|| {
      let _ = Context::default()
        .set_oracle_status(true)
        .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
        .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
        .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS);

      // SupportedMarketPairs is set to empty
      SupportedMarketPairs::<Test>::put(BoundedVec::try_from(Vec::new()).unwrap());

      let tdfy_temp = MarketPair {
        base_asset: CurrencyId::Tdfy,
        quote_asset: TEMP_CURRENCY_ID,
      };

      // Add TDFY_TEMP into supported market pairs
      assert_ok!(Oracle::add_market_pair(
        RuntimeOrigin::root(),
        tdfy_temp.clone(),
      ));

      // Event is emitted
      System::assert_has_event(MockEvent::Oracle(Event::MarketPairAdded {
        market_pair: tdfy_temp.clone(),
      }));

      // Market pair is added to the storage
      assert!(Oracle::supported_market_pairs().contains(&tdfy_temp));
    });
  }

  mod fails_when {
    use super::*;

    #[test]
    fn market_pair_is_already_supported() {
      new_test_ext().execute_with(|| {
        Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        assert_noop!(
          Oracle::add_market_pair(
            RuntimeOrigin::root(),
            MarketPair {
              base_asset: CurrencyId::Tdfy,
              quote_asset: TEMP_CURRENCY_ID,
            },
          ),
          Error::<Test>::MarketPairAlreadySupported
        );
      });
    }

    #[test]
    fn not_called_by_root_user() {
      new_test_ext().execute_with(|| {
        Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        assert_noop!(
          Oracle::add_market_pair(
            RuntimeOrigin::signed(ALICE_ACCOUNT_ID),
            MarketPair {
              base_asset: CurrencyId::Tdfy,
              quote_asset: TEMP_CURRENCY_ID,
            },
          ),
          BadOrigin
        );
      });
    }
  }
}

mod remove_market_pair {
  use super::*;

  #[test]
  fn succeed() {
    new_test_ext().execute_with(|| {
      let _ = Context::default()
        .set_oracle_status(true)
        .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
        .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
        .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS);

      let tdfy_temp = MarketPair {
        base_asset: CurrencyId::Tdfy,
        quote_asset: TEMP_CURRENCY_ID,
      };

      // Remove TDFY_TEMP from supported market pairs
      assert_ok!(Oracle::remove_market_pair(
        RuntimeOrigin::root(),
        tdfy_temp.clone(),
      ));

      // Event is emitted
      System::assert_has_event(MockEvent::Oracle(Event::MarketPairRemoved {
        market_pair: tdfy_temp.clone(),
      }));

      // Market pair is removed from the storage
      assert!(!Oracle::supported_market_pairs().contains(&tdfy_temp));
    });
  }

  mod fails_when {
    use super::*;

    #[test]
    fn market_pair_is_not_found() {
      new_test_ext().execute_with(|| {
        Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        assert_noop!(
          Oracle::remove_market_pair(
            RuntimeOrigin::root(),
            MarketPair {
              base_asset: CurrencyId::Tdfy,
              quote_asset: CurrencyId::Wrapped(100),
            },
          ),
          Error::<Test>::MarketPairNotSupported
        );
      });
    }

    #[test]
    fn not_called_by_root_user() {
      new_test_ext().execute_with(|| {
        Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, INITIAL_20_TDFYS)
          .mint_temp(CHARLIE_ACCOUNT_ID, INITIAL_10000_TEMPS);

        assert_noop!(
          Oracle::remove_market_pair(
            RuntimeOrigin::signed(ALICE_ACCOUNT_ID),
            MarketPair {
              base_asset: CurrencyId::Tdfy,
              quote_asset: TEMP_CURRENCY_ID,
            },
          ),
          BadOrigin
        );
      });
    }
  }
}
