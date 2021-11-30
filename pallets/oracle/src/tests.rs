use crate::{
  mock::{new_test_ext, AccountId, Adapter, Assets, Fees, Oracle, Origin, Test},
  pallet::*,
};
use frame_support::{
  assert_noop, assert_ok,
  traits::fungibles::{Inspect, Mutate},
};
use std::str::FromStr;
use tidefi_primitives::{
  pallet::{FeesExt, OracleExt},
  CurrencyId, Hash, TradeConfirmation, TradeStatus,
};

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    assert_eq!(Oracle::status(), false);
  });
}

#[test]
pub fn set_operational_status_works() {
  new_test_ext().execute_with(|| {
    let alice = Origin::signed(1u64.into());
    let bob = Origin::signed(2u64.into());
    assert_ok!(Oracle::set_status(alice.clone(), true));
    assert_noop!(Oracle::set_status(bob, false), Error::<Test>::AccessDenied);
    assert_eq!(Oracle::status(), true);
    assert_ok!(Oracle::set_status(alice, false));
    assert_eq!(Oracle::status(), false);
  });
}

#[test]
pub fn confirm_trade_partial_filling() {
  new_test_ext().execute_with(|| {
    let alice = Origin::signed(1u64.into());

    let temp_asset_id = 1;

    assert_ok!(Oracle::set_status(alice.clone(), true));
    assert_eq!(Oracle::status(), true);

    // add 1 tide to alice & all MMs
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &1u64.into(),
      1_000_000_000_000
    ));

    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &3u64.into(),
      1_000_000_000_000
    ));

    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &4u64.into(),
      1_000_000_000_000
    ));

    // add 20 tides to bob
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &2u64.into(),
      20_000_000_000_000
    ));

    // create TEMP asset
    assert_ok!(Assets::force_create(
      Origin::root(),
      temp_asset_id,
      1u64.into(),
      true,
      1
    ));

    // make TEMP asset as 2 decimals
    assert_ok!(Assets::set_metadata(
      alice.clone(),
      temp_asset_id,
      "TEMP".into(),
      "TEMP".into(),
      2
    ));

    // mint TEMP funds to the MMs
    assert_ok!(Assets::mint(
      alice.clone(),
      temp_asset_id,
      3u64.into(),
      1_000_000
    ));

    assert_ok!(Assets::mint(
      alice.clone(),
      temp_asset_id,
      4u64.into(),
      1_000_000
    ));

    // BOB: 10 TIDE for 200 TEMP (20 TEMP/TIDE)
    let (trade_request_id, trade_request) = Oracle::add_new_trade_in_queue(
      2u64.into(),
      CurrencyId::Tide,
      10_000_000_000_000,
      CurrencyId::Wrapped(temp_asset_id),
      20000,
    );

    // CHARLIE (MM): 4000 TEMP FOR 200 TIDE
    let (trade_request_mm_id, trade_request_mm) = Oracle::add_new_trade_in_queue(
      3u64.into(),
      CurrencyId::Wrapped(temp_asset_id),
      400000,
      CurrencyId::Tide,
      200_000_000_000_000,
    );

    // DAVE (MM): 8000 TEMP for 400 TIDE
    let (trade_request_mm2_id, trade_request_mm2) = Oracle::add_new_trade_in_queue(
      4u64.into(),
      CurrencyId::Wrapped(temp_asset_id),
      800_000,
      CurrencyId::Tide,
      400_000_000_000_000,
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

    assert_eq!(trade_request.status, TradeStatus::Pending);
    assert_eq!(trade_request_mm.status, TradeStatus::Pending);
    assert_eq!(trade_request_mm2.status, TradeStatus::Pending);

    assert_eq!(trade_request.block_number, 0);
    assert_eq!(trade_request_mm.block_number, 0);
    assert_eq!(trade_request_mm2.block_number, 0);

    // partial filling
    assert_ok!(Oracle::confirm_trade(
      alice.clone(),
      trade_request_id,
      vec![
        // charlie
        TradeConfirmation {
          request_id: trade_request_mm_id,
          // 5 tide
          amount_to_receive: 5_000_000_000_000,
          // 100 TEMP
          amount_to_send: 10_000,
        },
      ],
    ));

    // BOB: make sure the CLIENT current trade is partially filled and correctly updated
    let trade_request_filled = Oracle::trades(trade_request_id).unwrap();
    assert_eq!(trade_request_filled.status, TradeStatus::PartiallyFilled);
    // 5 tide
    assert_eq!(trade_request_filled.amount_from_filled, 5_000_000_000_000);
    // 100 TEMP
    assert_eq!(trade_request_filled.amount_to_filled, 10_000);

    // CHARLIE: make sure the MM current trade is partially filled and correctly updated
    let trade_request_filled = Oracle::trades(trade_request_mm_id).unwrap();
    assert_eq!(trade_request_filled.status, TradeStatus::PartiallyFilled);
    // 100 TEMP
    assert_eq!(trade_request_filled.amount_from_filled, 10_000);
    // 5 tide
    assert_eq!(trade_request_filled.amount_to_filled, 5_000_000_000_000);

    // another partial filling who should close the trade
    assert_ok!(Oracle::confirm_trade(
      alice.clone(),
      trade_request_id,
      vec![
        // dave
        TradeConfirmation {
          request_id: trade_request_mm2_id,
          // 5 tide
          amount_to_receive: 5_000_000_000_000,
          // 100 TEMP
          amount_to_send: 10_000,
        },
      ],
    ));

    // BOB: make sure the CLIENT current trade is totally filled (completed)
    let trade_request_filled = Oracle::trades(trade_request_id).unwrap();
    assert_eq!(trade_request_filled.status, TradeStatus::Completed);

    // cant send another trade confirmation as the request should be marked as completed
    // we do expect `InvalidRequestStatus`
    assert_noop!(
      Oracle::confirm_trade(alice, trade_request_id, vec![],),
      Error::<Test>::InvalidRequestStatus
    );

    // 10 tide
    assert_eq!(trade_request_filled.amount_from_filled, 10_000_000_000_000);
    // 200 TEMP
    assert_eq!(trade_request_filled.amount_to_filled, 20_000);

    // DAVE: make sure the MM current trade is partially filled and correctly updated
    let trade_request_filled = Oracle::trades(trade_request_mm2_id).unwrap();
    assert_eq!(trade_request_filled.status, TradeStatus::PartiallyFilled);
    // 100 TEMP
    assert_eq!(trade_request_filled.amount_from_filled, 10_000);
    // 5 tide
    assert_eq!(trade_request_filled.amount_to_filled, 5_000_000_000_000);
  });
}

#[test]
pub fn confirm_trade_simple_with_fees() {
  new_test_ext().execute_with(|| {
    let alice = Origin::signed(1u64.into());

    let temp_asset_id = 1;

    assert_ok!(Oracle::set_status(alice.clone(), true));
    assert_eq!(Oracle::status(), true);
    Fees::start_era();
    assert_eq!(Fees::active_era().is_none(), false);

    // add 1 tide to alice & mm
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &1u64.into(),
      1_000_000_000_000
    ));

    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &3u64.into(),
      1_000_000_000_000
    ));

    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &4u64.into(),
      1_000_000_000_000
    ));

    // add 1 tide to fees pallet account
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &Fees::account_id(),
      1_000_000_000_000
    ));

    // add 20 tides to bob
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &2u64.into(),
      20_000_000_000_000
    ));

    // create TEMP asset
    assert_ok!(Assets::force_create(
      Origin::root(),
      temp_asset_id,
      1u64.into(),
      true,
      1
    ));

    // make TEMP asset as 2 decimals
    assert_ok!(Assets::set_metadata(
      alice.clone(),
      temp_asset_id,
      "TEMP".into(),
      "TEMP".into(),
      2
    ));

    // mint TEMP funds to the MMs
    assert_ok!(Assets::mint(
      alice.clone(),
      temp_asset_id,
      3u64.into(),
      1_000_000
    ));
    assert_ok!(Assets::mint(
      alice.clone(),
      temp_asset_id,
      4u64.into(),
      1_000_000
    ));

    // BOB: 10 TIDE for 200 TEMP (20 TEMP/TIDE)
    let (trade_request_id, trade_request) = Oracle::add_new_trade_in_queue(
      2u64.into(),
      CurrencyId::Tide,
      10_000_000_000_000,
      CurrencyId::Wrapped(temp_asset_id),
      20_000,
    );

    // CHARLIE (MM): 4000 TEMP FOR 200 TIDE
    let (trade_request_mm_id, trade_request_mm) = Oracle::add_new_trade_in_queue(
      3u64.into(),
      CurrencyId::Wrapped(temp_asset_id),
      400_000,
      CurrencyId::Tide,
      200_000_000_000_000,
    );

    // DAVE (MM): 100 TEMP for 5 TIDE
    let (trade_request_mm2_id, trade_request_mm2) = Oracle::add_new_trade_in_queue(
      4u64.into(),
      CurrencyId::Wrapped(temp_asset_id),
      10_000,
      CurrencyId::Tide,
      5_000_000_000_000,
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

    assert_eq!(trade_request.status, TradeStatus::Pending);
    assert_eq!(trade_request_mm.status, TradeStatus::Pending);
    assert_eq!(trade_request_mm2.status, TradeStatus::Pending);

    assert_eq!(trade_request.block_number, 0);
    assert_eq!(trade_request_mm.block_number, 0);

    // partial filling
    assert_ok!(Oracle::confirm_trade(
      alice,
      trade_request_id,
      vec![
        // charlie
        TradeConfirmation {
          request_id: trade_request_mm_id,
          // 5 tide
          amount_to_receive: 5_000_000_000_000,
          // 100 TEMP
          amount_to_send: 10_000,
        },
        // dave
        TradeConfirmation {
          request_id: trade_request_mm2_id,
          // 5 tide
          amount_to_receive: 5_000_000_000_000,
          // 100 TEMP
          amount_to_send: 10_000,
        },
      ],
    ));

    // BOB: make sure the CLIENT current trade is totally filled (completed)
    let trade_request_filled = Oracle::trades(trade_request_id).unwrap();
    assert_eq!(trade_request_filled.status, TradeStatus::Completed);
    // 10 tide
    assert_eq!(trade_request_filled.amount_from_filled, 10_000_000_000_000);
    // 200 TEMP
    assert_eq!(trade_request_filled.amount_to_filled, 20_000);

    // CHARLIE: make sure the MM current trade is partially filled and correctly updated
    let trade_request_filled = Oracle::trades(trade_request_mm_id).unwrap();
    assert_eq!(trade_request_filled.status, TradeStatus::PartiallyFilled);
    // 100 TEMP
    assert_eq!(trade_request_filled.amount_from_filled, 10_000);
    // 5 tide
    assert_eq!(trade_request_filled.amount_to_filled, 5_000_000_000_000);

    // DAVE: make sure the MM current trade is totally filled (completed)
    let trade_request_filled = Oracle::trades(trade_request_mm2_id).unwrap();
    assert_eq!(trade_request_filled.status, TradeStatus::Completed);
    // 100 TEMP
    assert_eq!(trade_request_filled.amount_from_filled, 10_000);

    // make sure all source tokens are filled as there is a slippage on the destination
    assert_eq!(
      trade_request_filled.amount_from_filled,
      trade_request_filled.amount_from
    );
    // 5 tide
    assert_eq!(trade_request_filled.amount_to_filled, 5_000_000_000_000);

    // make sure all balances match
    assert_eq!(
      Adapter::balance(CurrencyId::Tide, &Fees::account_id()),
      // we burned 1 tide on start so it should contain 1.2 tide now
      1_200_000_000_000
    );

    assert_eq!(
      Adapter::balance(CurrencyId::Wrapped(temp_asset_id), &Fees::account_id()),
      400
    );

    // BOB Should have 10 tide remaining (started with 20)
    assert_eq!(
      Adapter::balance(CurrencyId::Tide, &2u64.into()),
      10_000_000_000_000
    );

    // BOB Should have 196 TEMP (2% fee)
    assert_eq!(
      Adapter::balance(CurrencyId::Wrapped(temp_asset_id), &2u64.into()),
      19600
    );

    // make sure fees are registered on chain
    let bob_fee = Fees::account_fees(CurrencyId::Tide, AccountId(2u64));
    assert_eq!(bob_fee.fee, 200_000_000_000);
    assert_eq!(bob_fee.amount, 10_000_000_000_000);

    let charlie_fee = Fees::account_fees(CurrencyId::Wrapped(temp_asset_id), AccountId(3u64));
    assert_eq!(charlie_fee.fee, 200);
    assert_eq!(charlie_fee.amount, 10_000);

    let dave_fee = Fees::account_fees(CurrencyId::Wrapped(temp_asset_id), AccountId(4u64));
    assert_eq!(dave_fee.fee, 200);
    assert_eq!(dave_fee.amount, 10_000);
  });
}
