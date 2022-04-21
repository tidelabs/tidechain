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
    new_test_ext, Adapter, Assets, Event as MockEvent, FeeAmount, Fees, MarketMakerFeeAmount,
    Oracle, Origin, System, Test,
  },
  pallet::*,
};
use frame_support::{
  assert_noop, assert_ok,
  traits::fungibles::{Inspect, InspectHold, Mutate},
};
use sp_runtime::{traits::Zero, Permill};
use std::str::FromStr;
use tidefi_primitives::{
  pallet::{FeesExt, OracleExt},
  Balance, CurrencyId, Hash, SwapConfirmation, SwapStatus, SwapType,
};

// TEMP Asset
const TEMP_ASSET_ID: u32 = 4;
const TEMP_CURRENCY_ID: CurrencyId = CurrencyId::Wrapped(TEMP_ASSET_ID);
const TEMP_ASSET_IS_SUFFICIENT: bool = true;
const TEMP_ASSET_MIN_BALANCE: u128 = 1;

// TEMP Asset Metadata
const TEMP_ASSET_NAME: &str = "TEMP";
const TEMP_ASSET_SYMBOL: &str = "TEMP";
const TEMP_ASSET_NUMBER_OF_DECIMAL_PLACES: u8 = 2;

// Asset Units
const ONE_TEMP: u128 = 100;
const ONE_MILLI_TIFI: u128 = 1_000_000_000;
const ONE_TIFI: u128 = 1_000 * ONE_MILLI_TIFI;

// Test Accounts
const ALICE_ACCOUNT_ID: u64 = 1;
const BOB_ACCOUNT_ID: u64 = 2;
const CHARLIE_ACCOUNT_ID: u64 = 3;
const DAVE_ACCOUNT_ID: u64 = 4;

// Extrinsic Hashes
const EXTRINSIC_HASH_0: [u8; 32] = [0; 32];
const EXTRINSIC_HASH_1: [u8; 32] = [1; 32];
const EXTRINSIC_HASH_2: [u8; 32] = [2; 32];

struct Context {
  alice: Origin,
  bob: Origin,
}

impl Default for Context {
  fn default() -> Self {
    Self {
      alice: Origin::signed(ALICE_ACCOUNT_ID),
      bob: Origin::signed(BOB_ACCOUNT_ID),
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

  fn create_temp_asset_and_metadata(self) -> Self {
    let temp_asset_owner = ALICE_ACCOUNT_ID;

    assert_ok!(Assets::force_create(
      Origin::root(),
      TEMP_ASSET_ID,
      temp_asset_owner,
      TEMP_ASSET_IS_SUFFICIENT,
      TEMP_ASSET_MIN_BALANCE
    ));

    assert_ok!(Assets::set_metadata(
      Origin::signed(temp_asset_owner),
      TEMP_ASSET_ID,
      TEMP_ASSET_NAME.into(),
      TEMP_ASSET_SYMBOL.into(),
      TEMP_ASSET_NUMBER_OF_DECIMAL_PLACES
    ));

    self
  }

  fn mint_tifi(self, account: u64, amount: u128) -> Self {
    Self::mint_asset_for_accounts(vec![account], CurrencyId::Tifi, amount);
    self
  }

  fn mint_temp(self, account: u64, amount: u128) -> Self {
    Self::mint_asset_for_accounts(vec![account], TEMP_CURRENCY_ID, amount);
    self
  }

  fn mint_asset_for_accounts(accounts: Vec<u64>, asset: CurrencyId, amount: u128) {
    for account in accounts {
      assert_ok!(Adapter::mint_into(asset, &account, amount));
    }
  }
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
pub fn confirm_swap_partial_filling() {
  new_test_ext().execute_with(|| {
    const BOB_INITIAL_BALANCE: Balance = 20 * ONE_TIFI;
    const CHARLIE_INITIAL_WRAPPED_BALANCE: Balance = 10_000 * ONE_TEMP;
    const DAVE_INITIAL_WRAPPED_BALANCE: Balance = 10_000 * ONE_TEMP;
    const CHARLIE_INITIAL_TRADE: Balance = 4_000 * ONE_TEMP;

    let context = Context::default()
      .set_oracle_status(true)
      .mint_tifi(ALICE_ACCOUNT_ID, ONE_TIFI)
      .mint_tifi(CHARLIE_ACCOUNT_ID, ONE_TIFI)
      .mint_tifi(DAVE_ACCOUNT_ID, ONE_TIFI)
      .mint_tifi(BOB_ACCOUNT_ID, BOB_INITIAL_BALANCE)
      .create_temp_asset_and_metadata()
      .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_WRAPPED_BALANCE)
      .mint_temp(DAVE_ACCOUNT_ID, DAVE_INITIAL_WRAPPED_BALANCE);

    assert_eq!(Fees::account_id(), 8246216774960574317);

    assert_eq!(
      Adapter::balance(TEMP_CURRENCY_ID, &CHARLIE_ACCOUNT_ID),
      CHARLIE_INITIAL_WRAPPED_BALANCE
    );

    // BOB: 10 TIFI for 200 TEMP (20 TEMP/TIFI)
    let bob_initial_trade: Balance = 10 * ONE_TIFI;

    let (trade_request_id, trade_request) = Oracle::add_new_swap_in_queue(
      BOB_ACCOUNT_ID,
      CurrencyId::Tifi,
      bob_initial_trade,
      TEMP_CURRENCY_ID,
      200 * ONE_TEMP,
      0,
      EXTRINSIC_HASH_0,
      false,
      SwapType::Limit,
      // 2% slippage tolerance
      Permill::from_percent(2),
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Tifi, &BOB_ACCOUNT_ID),
      bob_initial_trade
        // add 0.2% fee
        .saturating_add(FeeAmount::get() * bob_initial_trade)
    );

    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Tifi, &BOB_ACCOUNT_ID, true),
      BOB_INITIAL_BALANCE
        // reduce 2% slippage
        .saturating_sub(bob_initial_trade)
        // reduce 0.2% network fee
        .saturating_sub(FeeAmount::get() * bob_initial_trade)
    );

    // CHARLIE (MM): 4000 TEMP FOR 200 TIFI
    let (trade_request_mm_id, trade_request_mm) = Oracle::add_new_swap_in_queue(
      CHARLIE_ACCOUNT_ID,
      TEMP_CURRENCY_ID,
      CHARLIE_INITIAL_TRADE,
      CurrencyId::Tifi,
      200 * ONE_TIFI,
      0,
      EXTRINSIC_HASH_1,
      true,
      SwapType::Limit,
      // 4% slippage tolerance
      Permill::from_percent(4),
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(TEMP_CURRENCY_ID, &CHARLIE_ACCOUNT_ID),
      CHARLIE_INITIAL_TRADE
        // add 0.1% fee
        .saturating_add(MarketMakerFeeAmount::get() * CHARLIE_INITIAL_TRADE)
    );

    let last_charlie_balance = CHARLIE_INITIAL_WRAPPED_BALANCE
      .saturating_sub(CHARLIE_INITIAL_TRADE)
      .saturating_sub(MarketMakerFeeAmount::get() * CHARLIE_INITIAL_TRADE);

    assert_eq!(
      Adapter::balance(TEMP_CURRENCY_ID, &CHARLIE_ACCOUNT_ID),
      last_charlie_balance
    );

    assert_eq!(
      Adapter::reducible_balance(TEMP_CURRENCY_ID, &CHARLIE_ACCOUNT_ID, true),
      CHARLIE_INITIAL_WRAPPED_BALANCE
        // keep-alive token
        .saturating_sub(1_u128)
        // slippage
        .saturating_sub(CHARLIE_INITIAL_TRADE)
        // fees
        .saturating_sub(MarketMakerFeeAmount::get() * CHARLIE_INITIAL_TRADE)
    );

    assert_eq!(
      Adapter::reducible_balance(TEMP_CURRENCY_ID, &CHARLIE_ACCOUNT_ID, false),
      // minted 10_000 TEMP on genesis (no keep-alive)
      CHARLIE_INITIAL_WRAPPED_BALANCE
        // slippage
        .saturating_sub(CHARLIE_INITIAL_TRADE)
        // fees
        .saturating_sub(MarketMakerFeeAmount::get() * CHARLIE_INITIAL_TRADE)
    );

    // DAVE (MM): 8000 TEMP for 400 TIFI
    let dave_initial_trade: Balance = 8_000 * ONE_TEMP;

    let (trade_request_mm2_id, trade_request_mm2) = Oracle::add_new_swap_in_queue(
      DAVE_ACCOUNT_ID,
      TEMP_CURRENCY_ID,
      8_000 * ONE_TEMP,
      CurrencyId::Tifi,
      400 * ONE_TIFI,
      0,
      EXTRINSIC_HASH_2,
      true,
      SwapType::Limit,
      // 5% slippage tolerance
      Permill::from_percent(5),
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(TEMP_CURRENCY_ID, &DAVE_ACCOUNT_ID),
      dave_initial_trade
        // add 0.1% fee
        .saturating_add(MarketMakerFeeAmount::get() * dave_initial_trade)
    );

    // make sure our trade request is created correctly
    assert_eq!(
      trade_request_id,
      Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
        .unwrap_or_default()
    );
    assert_eq!(trade_request.block_number, 0);

    assert_eq!(
      trade_request_mm_id,
      Hash::from_str("0x9ee76e89d3eae9ddad2e0b731e29ddcfa0781f7035600c5eb885637592e1d2c2")
        .unwrap_or_default()
    );

    assert_eq!(trade_request.status, SwapStatus::Pending);
    assert_eq!(trade_request_mm.status, SwapStatus::Pending);
    assert_eq!(trade_request_mm2.status, SwapStatus::Pending);

    assert_eq!(trade_request.block_number, 0);
    assert_eq!(trade_request_mm.block_number, 0);
    assert_eq!(trade_request_mm2.block_number, 0);

    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &BOB_ACCOUNT_ID),
      20 * ONE_TIFI
    );

    let partial_filling_amount_charlie: Balance = 100 * ONE_TEMP;
    // partial filling
    assert_ok!(Oracle::confirm_swap(
      context.alice.clone(),
      trade_request_id,
      vec![
        // charlie
        SwapConfirmation {
          request_id: trade_request_mm_id,
          // 5 tifi
          amount_to_receive: 5 * ONE_TIFI,
          // 100 TEMP
          amount_to_send: partial_filling_amount_charlie,
        },
      ],
    ));

    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &BOB_ACCOUNT_ID),
      BOB_INITIAL_BALANCE
        // reduce 2% slippage
        .saturating_sub(5 * ONE_TIFI)
        // reduce 0.2% network fee
        .saturating_sub(FeeAmount::get() * (5 * ONE_TIFI))
    );

    // swap confirmation for bob (user)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_id,
      status: SwapStatus::PartiallyFilled,
      account_id: BOB_ACCOUNT_ID,
      currency_from: CurrencyId::Tifi,
      currency_amount_from: 5 * ONE_TIFI,
      currency_to: TEMP_CURRENCY_ID,
      currency_amount_to: partial_filling_amount_charlie,
      initial_extrinsic_hash: EXTRINSIC_HASH_0,
    }));

    // swap confirmation for charlie (mm)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_mm_id,
      status: SwapStatus::PartiallyFilled,
      account_id: CHARLIE_ACCOUNT_ID,
      currency_from: TEMP_CURRENCY_ID,
      currency_amount_from: partial_filling_amount_charlie,
      currency_to: CurrencyId::Tifi,
      currency_amount_to: 5 * ONE_TIFI,
      initial_extrinsic_hash: EXTRINSIC_HASH_1,
    }));

    // BOB: make sure the CLIENT current trade is partially filled and correctly updated
    let trade_request_filled = Oracle::swaps(trade_request_id).unwrap();
    assert_eq!(trade_request_filled.status, SwapStatus::PartiallyFilled);

    let trade_request_account = Oracle::account_swaps(BOB_ACCOUNT_ID).unwrap();
    assert_eq!(
      trade_request_account
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_id),
      Some(&(trade_request_id, SwapStatus::PartiallyFilled))
    );

    // 5 tifi
    assert_eq!(trade_request_filled.amount_from_filled, 5 * ONE_TIFI);
    // 100 TEMP
    assert_eq!(
      trade_request_filled.amount_to_filled,
      partial_filling_amount_charlie
    );

    // CHARLIE: make sure the MM current trade is partially filled and correctly updated
    let trade_request_filled = Oracle::swaps(trade_request_mm_id).unwrap();
    assert_eq!(trade_request_filled.status, SwapStatus::PartiallyFilled);
    // 100 TEMP
    assert_eq!(
      trade_request_filled.amount_from_filled,
      partial_filling_amount_charlie
    );
    // 5 tifi
    assert_eq!(trade_request_filled.amount_to_filled, 5 * ONE_TIFI);

    // another partial filling who should close the trade
    assert_ok!(Oracle::confirm_swap(
      context.alice.clone(),
      trade_request_id,
      vec![
        // dave
        SwapConfirmation {
          request_id: trade_request_mm2_id,
          // 5 tifi
          amount_to_receive: 5 * ONE_TIFI,
          // 100 TEMP
          amount_to_send: partial_filling_amount_charlie,
        },
      ],
    ));

    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &BOB_ACCOUNT_ID),
      BOB_INITIAL_BALANCE
        .saturating_sub(10 * ONE_TIFI)
        // reduce 0.2% network fee
        .saturating_sub(FeeAmount::get() * (10 * ONE_TIFI))
    );

    // swap confirmation for bob (user)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_id,
      status: SwapStatus::Completed,
      account_id: BOB_ACCOUNT_ID,
      currency_from: CurrencyId::Tifi,
      currency_amount_from: 5 * ONE_TIFI,
      currency_to: TEMP_CURRENCY_ID,
      currency_amount_to: partial_filling_amount_charlie,
      initial_extrinsic_hash: EXTRINSIC_HASH_0,
    }));

    // swap confirmation for dave (second mm)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_mm2_id,
      status: SwapStatus::PartiallyFilled,
      account_id: DAVE_ACCOUNT_ID,
      currency_from: TEMP_CURRENCY_ID,
      currency_amount_from: partial_filling_amount_charlie,
      currency_to: CurrencyId::Tifi,
      currency_amount_to: 5 * ONE_TIFI,
      initial_extrinsic_hash: EXTRINSIC_HASH_2,
    }));

    // BOB: make sure the CLIENT current trade is deleted
    assert!(Oracle::swaps(trade_request_id).is_none());
    let trade_request_account = Oracle::account_swaps(BOB_ACCOUNT_ID).unwrap();
    assert_eq!(
      trade_request_account
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_id),
      None
    );

    // cant send another trade confirmation as the request should be deleted
    // we do expect `InvalidRequestId`
    assert_noop!(
      Oracle::confirm_swap(context.alice.clone(), trade_request_id, vec![],),
      Error::<Test>::InvalidRequestId
    );

    // DAVE: make sure the MM current trade is partially filled and correctly updated
    let trade_request_filled = Oracle::swaps(trade_request_mm2_id).unwrap();
    assert_eq!(trade_request_filled.status, SwapStatus::PartiallyFilled);
    // 100 TEMP
    assert_eq!(trade_request_filled.amount_from_filled, 100 * ONE_TEMP);
    // 5 tifi
    assert_eq!(trade_request_filled.amount_to_filled, 5 * ONE_TIFI);

    // cancel our mm's swap to release the funds
    assert_ok!(Oracle::cancel_swap(
      context.alice.clone(),
      trade_request_mm_id,
    ));
    assert_ok!(Oracle::cancel_swap(context.alice, trade_request_mm2_id,));

    // validate all balance
    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &BOB_ACCOUNT_ID),
      BOB_INITIAL_BALANCE
        .saturating_sub(10 * ONE_TIFI)
        // reduce 0.2% network fee
        .saturating_sub(FeeAmount::get() * (10 * ONE_TIFI))
    );
    assert_eq!(
      Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
      200 * ONE_TEMP
    );
    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Tifi, &BOB_ACCOUNT_ID),
      Zero::zero()
    );

    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &CHARLIE_ACCOUNT_ID),
      // swap + initial balance
      5 * ONE_TIFI + ONE_TIFI
    );

    assert_eq!(
      Adapter::reducible_balance(TEMP_CURRENCY_ID, &CHARLIE_ACCOUNT_ID, false),
      CHARLIE_INITIAL_WRAPPED_BALANCE
        .saturating_sub(partial_filling_amount_charlie)
        .saturating_sub(MarketMakerFeeAmount::get() * partial_filling_amount_charlie)
    );

    assert_eq!(
      Adapter::balance_on_hold(TEMP_CURRENCY_ID, &CHARLIE_ACCOUNT_ID),
      Zero::zero()
    );

    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &DAVE_ACCOUNT_ID),
      // swap + initial balance
      5 * ONE_TIFI + ONE_TIFI
    );

    assert_eq!(
      Adapter::balance(TEMP_CURRENCY_ID, &DAVE_ACCOUNT_ID),
      DAVE_INITIAL_WRAPPED_BALANCE
        .saturating_sub(partial_filling_amount_charlie)
        .saturating_sub(MarketMakerFeeAmount::get() * partial_filling_amount_charlie)
    );

    assert_eq!(
      Adapter::balance_on_hold(TEMP_CURRENCY_ID, &DAVE_ACCOUNT_ID),
      Zero::zero()
    );
  });
}

#[test]
pub fn confirm_swap_with_fees() {
  new_test_ext().execute_with(|| {
    let context = Context::default()
      .set_oracle_status(true)
      .mint_tifi(ALICE_ACCOUNT_ID, ONE_TIFI)
      .mint_tifi(CHARLIE_ACCOUNT_ID, ONE_TIFI)
      .mint_tifi(DAVE_ACCOUNT_ID, ONE_TIFI)
      .mint_tifi(BOB_ACCOUNT_ID, 20 * ONE_TIFI)
      .create_temp_asset_and_metadata()
      .mint_temp(CHARLIE_ACCOUNT_ID, 10_000 * ONE_TEMP)
      .mint_temp(DAVE_ACCOUNT_ID, 10_000 * ONE_TEMP);

    Fees::start_era();
    assert!(!Fees::active_era().is_none());
    let current_era = Fees::active_era().unwrap().index;

    // BOB: 10 TIFI for 200 TEMP (20 TEMP/TIFI)
    let (trade_request_id, trade_request) = Oracle::add_new_swap_in_queue(
      BOB_ACCOUNT_ID,
      CurrencyId::Tifi,
      10 * ONE_TIFI,
      TEMP_CURRENCY_ID,
      200 * ONE_TEMP,
      0,
      EXTRINSIC_HASH_0,
      false,
      SwapType::Limit,
      // 2% slippage tolerance
      Permill::from_percent(2),
    )
    .unwrap();

    // CHARLIE (MM): 4000 TEMP FOR 200 TIFI
    let (trade_request_mm_id, trade_request_mm) = Oracle::add_new_swap_in_queue(
      CHARLIE_ACCOUNT_ID,
      TEMP_CURRENCY_ID,
      4_000 * ONE_TEMP,
      CurrencyId::Tifi,
      200 * ONE_TIFI,
      0,
      EXTRINSIC_HASH_1,
      true,
      SwapType::Limit,
      // 5% slippage tolerance
      Permill::from_percent(5),
    )
    .unwrap();

    // DAVE (MM): 100 TEMP for 5 TIFI
    let (trade_request_mm2_id, trade_request_mm2) = Oracle::add_new_swap_in_queue(
      DAVE_ACCOUNT_ID,
      TEMP_CURRENCY_ID,
      100 * ONE_TEMP,
      CurrencyId::Tifi,
      5 * ONE_TIFI,
      0,
      EXTRINSIC_HASH_2,
      true,
      SwapType::Limit,
      // 4% slippage tolerance
      Permill::from_percent(4),
    )
    .unwrap();

    // make sure our trade request is created correctly
    assert_eq!(
      trade_request_id,
      Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
        .unwrap_or_default()
    );
    assert_eq!(trade_request.block_number, 0);

    assert_eq!(
      trade_request_mm_id,
      Hash::from_str("0x9ee76e89d3eae9ddad2e0b731e29ddcfa0781f7035600c5eb885637592e1d2c2")
        .unwrap_or_default()
    );

    assert_eq!(trade_request.status, SwapStatus::Pending);
    assert_eq!(trade_request_mm.status, SwapStatus::Pending);
    assert_eq!(trade_request_mm2.status, SwapStatus::Pending);

    assert_eq!(
      Oracle::account_swaps(BOB_ACCOUNT_ID)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_id),
      Some(&(trade_request_id, SwapStatus::Pending))
    );

    assert_eq!(
      Oracle::account_swaps(CHARLIE_ACCOUNT_ID)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_mm_id),
      Some(&(trade_request_mm_id, SwapStatus::Pending))
    );

    assert_eq!(
      Oracle::account_swaps(DAVE_ACCOUNT_ID)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_mm2_id),
      Some(&(trade_request_mm2_id, SwapStatus::Pending))
    );

    assert_eq!(trade_request.block_number, 0);
    assert_eq!(trade_request_mm.block_number, 0);

    // partial filling
    assert_ok!(Oracle::confirm_swap(
      context.alice,
      trade_request_id,
      vec![
        // charlie
        SwapConfirmation {
          request_id: trade_request_mm_id,
          // 5 tifi
          amount_to_receive: 5 * ONE_TIFI,
          // 100 TEMP
          amount_to_send: 100 * ONE_TEMP,
        },
        // dave
        SwapConfirmation {
          request_id: trade_request_mm2_id,
          // 5 tifi
          amount_to_receive: 5 * ONE_TIFI,
          // 100 TEMP
          amount_to_send: 100 * ONE_TEMP,
        },
      ],
    ));

    // swap confirmation for bob (user)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_id,
      status: SwapStatus::Completed,
      account_id: BOB_ACCOUNT_ID,
      currency_from: CurrencyId::Tifi,
      currency_amount_from: 10 * ONE_TIFI,
      currency_to: TEMP_CURRENCY_ID,
      currency_amount_to: 200 * ONE_TEMP,
      initial_extrinsic_hash: EXTRINSIC_HASH_0,
    }));

    // swap confirmation for charlie (mm1)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_mm_id,
      status: SwapStatus::PartiallyFilled,
      account_id: CHARLIE_ACCOUNT_ID,
      currency_from: TEMP_CURRENCY_ID,
      currency_amount_from: 100 * ONE_TEMP,
      currency_to: CurrencyId::Tifi,
      currency_amount_to: 5 * ONE_TIFI,
      initial_extrinsic_hash: EXTRINSIC_HASH_1,
    }));

    // swap confirmation for dave (mm2)
    // the trade should be closed, because amount_from of the request is filled
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_mm2_id,
      status: SwapStatus::Completed,
      account_id: DAVE_ACCOUNT_ID,
      currency_from: TEMP_CURRENCY_ID,
      currency_amount_from: 100 * ONE_TEMP,
      currency_to: CurrencyId::Tifi,
      currency_amount_to: 5 * ONE_TIFI,
      initial_extrinsic_hash: EXTRINSIC_HASH_2,
    }));

    // BOB: make sure the CLIENT current trade is deleted
    assert!(Oracle::swaps(trade_request_id).is_none());
    assert_eq!(
      Oracle::account_swaps(BOB_ACCOUNT_ID)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_id),
      None
    );

    // CHARLIE: make sure the MM current trade is partially filled and correctly updated
    assert_eq!(
      Oracle::account_swaps(CHARLIE_ACCOUNT_ID)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_mm_id),
      Some(&(trade_request_mm_id, SwapStatus::PartiallyFilled))
    );

    let trade_request_filled = Oracle::swaps(trade_request_mm_id).unwrap();
    assert_eq!(trade_request_filled.status, SwapStatus::PartiallyFilled);
    // 100 TEMP
    assert_eq!(trade_request_filled.amount_from_filled, 100 * ONE_TEMP);
    // 5 tifi
    assert_eq!(trade_request_filled.amount_to_filled, 5 * ONE_TIFI);

    // DAVE: make sure the MM current trade is totally filled (deleted)
    assert!(Oracle::swaps(trade_request_mm2_id).is_none());
    assert_eq!(
      Oracle::account_swaps(DAVE_ACCOUNT_ID)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_mm2_id),
      None
    );

    // make sure all balances match
    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &Fees::account_id()),
      // we burned 1 tifi on start so it should contain 1.2 tifi now
      1_200 * ONE_MILLI_TIFI
    );

    assert_eq!(
      Adapter::balance(TEMP_CURRENCY_ID, &Fees::account_id()),
      2 * ONE_TEMP
    );

    // BOB Should have 9.8 tifi remaining (started with 20), sent 10 tifi and paid 2% fees
    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &BOB_ACCOUNT_ID),
      9_800 * ONE_MILLI_TIFI
    );

    // BOB Should have 200 TEMP
    assert_eq!(
      Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
      200 * ONE_TEMP
    );

    // make sure fees are registered on chain
    let bob_fee = Fees::account_fees(current_era, BOB_ACCOUNT_ID);
    assert_eq!(bob_fee.first().unwrap().1.fee, 200 * ONE_MILLI_TIFI);
    assert_eq!(bob_fee.first().unwrap().1.amount, 10 * ONE_TIFI);

    let charlie_fee = Fees::account_fees(current_era, CHARLIE_ACCOUNT_ID);
    assert_eq!(charlie_fee.first().unwrap().1.fee, 100);
    assert_eq!(charlie_fee.first().unwrap().1.amount, 10_000);

    let dave_fee = Fees::account_fees(current_era, DAVE_ACCOUNT_ID);
    assert_eq!(dave_fee.first().unwrap().1.fee, 100);
    assert_eq!(dave_fee.first().unwrap().1.amount, 10_000);
  });
}

#[test]
pub fn confirm_swap_ourself() {
  new_test_ext().execute_with(|| {
    const BOB_INITIAL_BALANCE: Balance = 20 * ONE_TIFI;
    const BOB_INITIAL_WRAPPED_BALANCE: Balance = 10_000 * ONE_TEMP;

    let context = Context::default()
      .set_oracle_status(true)
      .mint_tifi(ALICE_ACCOUNT_ID, ONE_TIFI)
      .mint_tifi(BOB_ACCOUNT_ID, BOB_INITIAL_BALANCE)
      .create_temp_asset_and_metadata()
      .mint_temp(BOB_ACCOUNT_ID, BOB_INITIAL_WRAPPED_BALANCE);

    assert_eq!(Fees::account_id(), 8246216774960574317);

    assert_eq!(
      Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
      BOB_INITIAL_WRAPPED_BALANCE
    );

    // BOB: 10 TIFI for 200 TEMP (20 TEMP/TIFI)
    let bob_initial_trade: Balance = 10 * ONE_TIFI;

    let (trade_request_id, trade_request) = Oracle::add_new_swap_in_queue(
      BOB_ACCOUNT_ID,
      CurrencyId::Tifi,
      bob_initial_trade,
      TEMP_CURRENCY_ID,
      400 * ONE_TEMP,
      0,
      EXTRINSIC_HASH_0,
      false,
      SwapType::Limit,
      // 2% slippage tolerance
      Permill::from_percent(2),
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Tifi, &BOB_ACCOUNT_ID),
      bob_initial_trade
        // add 0.2% fee
        .saturating_add(FeeAmount::get() * bob_initial_trade)
    );

    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Tifi, &BOB_ACCOUNT_ID, true),
      BOB_INITIAL_BALANCE
        // reduce 2% slippage
        .saturating_sub(bob_initial_trade)
        // reduce 0.2% network fee
        .saturating_sub(FeeAmount::get() * bob_initial_trade)
    );

    let bob_initial_trade: Balance = 400 * ONE_TEMP;

    let (trade_request_mm_id, trade_request_mm) = Oracle::add_new_swap_in_queue(
      BOB_ACCOUNT_ID,
      TEMP_CURRENCY_ID,
      bob_initial_trade,
      CurrencyId::Tifi,
      10 * ONE_TIFI,
      0,
      EXTRINSIC_HASH_0,
      true,
      SwapType::Limit,
      // 5% slippage tolerance
      Permill::from_percent(5),
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
      bob_initial_trade
        // add 0.1% fee
        .saturating_add(MarketMakerFeeAmount::get() * bob_initial_trade)
    );

    // make sure our trade request is created correctly
    assert_eq!(
      trade_request_id,
      Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
        .unwrap_or_default()
    );
    assert_eq!(trade_request.block_number, 0);

    assert_eq!(
      trade_request_mm_id,
      Hash::from_str("0xe0424aac19ef997f1b76ac20d400aecc2ee0258d9eacb7013c3fcfa2e55bdc67")
        .unwrap_or_default()
    );

    assert_eq!(trade_request.status, SwapStatus::Pending);
    assert_eq!(trade_request_mm.status, SwapStatus::Pending);

    assert_eq!(trade_request.block_number, 0);
    assert_eq!(trade_request_mm.block_number, 0);

    // partial filling
    assert_ok!(Oracle::confirm_swap(
      context.alice.clone(),
      trade_request_id,
      vec![
        // charlie
        SwapConfirmation {
          request_id: trade_request_mm_id,
          amount_to_receive: 10 * ONE_TIFI,
          amount_to_send: 400 * ONE_TEMP,
        },
      ],
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
      Adapter::reducible_balance(CurrencyId::Tifi, &BOB_ACCOUNT_ID, false),
      // we should refund the extra fees paid on the slippage value
      BOB_INITIAL_BALANCE.saturating_sub(FeeAmount::get() * (10 * ONE_TIFI))
    );

    assert_eq!(
      Adapter::reducible_balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID, false),
      BOB_INITIAL_WRAPPED_BALANCE.saturating_sub(MarketMakerFeeAmount::get() * (400 * ONE_TEMP))
    );

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Tifi, &BOB_ACCOUNT_ID),
      Zero::zero()
    );
    assert_eq!(
      Adapter::balance_on_hold(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
      Zero::zero()
    );
  });
}

#[test]
pub fn test_slippage() {
  new_test_ext().execute_with(|| {
    const BOB_INITIAL_BALANCE: Balance = 20 * ONE_TIFI;
    const BOB_INITIAL_WRAPPED_BALANCE: Balance = 10_000 * ONE_TEMP;

    let context = Context::default()
      .set_oracle_status(true)
      .mint_tifi(ALICE_ACCOUNT_ID, ONE_TIFI)
      .mint_tifi(BOB_ACCOUNT_ID, BOB_INITIAL_BALANCE)
      .create_temp_asset_and_metadata()
      .mint_temp(BOB_ACCOUNT_ID, BOB_INITIAL_WRAPPED_BALANCE);

    assert_eq!(Fees::account_id(), 8246216774960574317);

    assert_eq!(
      Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
      BOB_INITIAL_WRAPPED_BALANCE
    );

    // BOB: 10 TIFI for 200 TEMP (20 TEMP/TIFI)
    let bob_initial_trade: Balance = 10 * ONE_TIFI;

    let (trade_request_id, trade_request) = Oracle::add_new_swap_in_queue(
      BOB_ACCOUNT_ID,
      CurrencyId::Tifi,
      bob_initial_trade,
      TEMP_CURRENCY_ID,
      400 * ONE_TEMP,
      0,
      EXTRINSIC_HASH_0,
      false,
      SwapType::Market,
      // 2% slippage tolerance
      Permill::from_percent(2),
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Tifi, &BOB_ACCOUNT_ID),
      bob_initial_trade
        // add 0.2% fee
        .saturating_add(FeeAmount::get() * bob_initial_trade)
    );

    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Tifi, &BOB_ACCOUNT_ID, true),
      BOB_INITIAL_BALANCE
        // reduce 2% slippage
        .saturating_sub(bob_initial_trade)
        // reduce 0.2% network fee
        .saturating_sub(FeeAmount::get() * bob_initial_trade)
    );

    let (trade_request_mm_id, trade_request_mm) = Oracle::add_new_swap_in_queue(
      BOB_ACCOUNT_ID,
      TEMP_CURRENCY_ID,
      // ratio is a bit different (mm is willing to pay a bit more for the same amount)
      500 * ONE_TEMP,
      CurrencyId::Tifi,
      bob_initial_trade,
      0,
      EXTRINSIC_HASH_0,
      true,
      SwapType::Limit,
      Permill::from_percent(0),
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
      (500 * ONE_TEMP)
        // add 0.1% fee
        .saturating_add(MarketMakerFeeAmount::get() * (500 * ONE_TEMP))
    );

    // make sure our trade request is created correctly
    assert_eq!(
      trade_request_id,
      Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
        .unwrap_or_default()
    );
    assert_eq!(trade_request.block_number, 0);

    assert_eq!(
      trade_request_mm_id,
      Hash::from_str("0xe0424aac19ef997f1b76ac20d400aecc2ee0258d9eacb7013c3fcfa2e55bdc67")
        .unwrap_or_default()
    );

    assert_eq!(trade_request.status, SwapStatus::Pending);
    assert_eq!(trade_request_mm.status, SwapStatus::Pending);

    assert_eq!(trade_request.block_number, 0);
    assert_eq!(trade_request_mm.block_number, 0);

    assert_noop!(
      Oracle::confirm_swap(
        context.alice.clone(),
        trade_request_id,
        vec![SwapConfirmation {
          request_id: trade_request_mm_id,
          amount_to_receive: 8 * ONE_TIFI,
          amount_to_send: 400 * ONE_TEMP,
        },],
      ),
      Error::<Test>::Overflow
    );

    // partial filling
    assert_ok!(Oracle::confirm_swap(
      context.alice.clone(),
      trade_request_id,
      vec![SwapConfirmation {
        request_id: trade_request_mm_id,
        // 9.8 test token (within the 2% slippage of the initial request)
        amount_to_receive: 9_800 * ONE_MILLI_TIFI,
        amount_to_send: 400 * ONE_TEMP,
      },],
    ));

    // market order got deleted
    assert!(Oracle::swaps(trade_request_id).is_none());
    // limit order isnt deleted as its not fully filled
    assert!(Oracle::swaps(trade_request_mm_id).is_some());
  });
}
