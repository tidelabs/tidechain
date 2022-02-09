use crate::{
  mock::{new_test_ext, Adapter, Assets, Fees, Oracle, Origin, Test},
  pallet::*,
};
use frame_support::{
  assert_noop, assert_ok,
  traits::fungibles::{Inspect, InspectHold, Mutate},
};
use sp_runtime::{traits::Zero, Percent};
use std::str::FromStr;
use tidefi_primitives::{
  pallet::{FeesExt, OracleExt},
  CurrencyId, Hash, SwapConfirmation, SwapStatus,
};

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    assert!(!Oracle::status());
  });
}

#[test]
pub fn set_operational_status_works() {
  new_test_ext().execute_with(|| {
    let alice = Origin::signed(1u64);
    let bob = Origin::signed(2u64);
    assert_ok!(Oracle::set_status(alice.clone(), true));
    assert_noop!(Oracle::set_status(bob, false), Error::<Test>::AccessDenied);
    assert!(Oracle::status());
    assert_ok!(Oracle::set_status(alice, false));
    assert!(!Oracle::status());
  });
}

#[test]
pub fn confirm_swap_partial_filling() {
  new_test_ext().execute_with(|| {
    let alice = Origin::signed(1u64);

    let temp_asset_id = 1;

    assert_ok!(Oracle::set_status(alice.clone(), true));
    assert!(Oracle::status());

    // set fee to 0%
    assert_ok!(Fees::set_fees_percentage(Origin::root(), Percent::zero()));

    // add 1 tide to alice & all MMs
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &1u64,
      1_000_000_000_000
    ));

    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &3u64,
      1_000_000_000_000
    ));

    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &4u64,
      1_000_000_000_000
    ));

    // add 20 tides to bob
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &2u64,
      20_000_000_000_000
    ));

    // create TEMP asset
    assert_ok!(Assets::force_create(
      Origin::root(),
      temp_asset_id,
      1u64,
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
    assert_ok!(Assets::mint(alice.clone(), temp_asset_id, 3u64, 1_000_000));
    assert_eq!(
      Adapter::balance(CurrencyId::Wrapped(temp_asset_id), &3u64),
      1_000_000
    );

    assert_ok!(Assets::mint(alice.clone(), temp_asset_id, 4u64, 1_000_000));

    // BOB: 10 TIDE for 200 TEMP (20 TEMP/TIDE)
    let (trade_request_id, trade_request) = Oracle::add_new_swap_in_queue(
      2u64,
      CurrencyId::Tide,
      10_000_000_000_000,
      CurrencyId::Wrapped(temp_asset_id),
      20_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Tide, &2u64),
      // we burned 1 tide on start so it should contain 1.2 tide now
      10_000_000_000_000
    );

    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Tide, &2u64, true),
      // minted 20_000_000_000_000 on genesis
      10_000_000_000_000
    );

    // CHARLIE (MM): 4000 TEMP FOR 200 TIDE
    let (trade_request_mm_id, trade_request_mm) = Oracle::add_new_swap_in_queue(
      3u64,
      CurrencyId::Wrapped(temp_asset_id),
      400_000,
      CurrencyId::Tide,
      200_000_000_000_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Wrapped(temp_asset_id), &3u64),
      400_000
    );
    assert_eq!(
      Adapter::balance(CurrencyId::Wrapped(temp_asset_id), &3u64),
      600_000
    );
    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Wrapped(temp_asset_id), &3u64, true),
      // minted 1_000_000 on genesis + 1 as keep alive cost
      599_999
    );
    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Wrapped(temp_asset_id), &3u64, false),
      // minted 1_000_000 on genesis (no keep-alive)
      600_000
    );

    // DAVE (MM): 8000 TEMP for 400 TIDE
    let (trade_request_mm2_id, trade_request_mm2) = Oracle::add_new_swap_in_queue(
      4u64,
      CurrencyId::Wrapped(temp_asset_id),
      800_000,
      CurrencyId::Tide,
      400_000_000_000_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Wrapped(temp_asset_id), &4u64),
      // we burned 1 tide on start so it should contain 1.2 tide now
      800_000
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

    // partial filling
    assert_ok!(Oracle::confirm_swap(
      alice.clone(),
      trade_request_id,
      vec![
        // charlie
        SwapConfirmation {
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
    assert_eq!(trade_request_filled.status, SwapStatus::PartiallyFilled);
    // 5 tide
    assert_eq!(trade_request_filled.amount_from_filled, 5_000_000_000_000);
    // 100 TEMP
    assert_eq!(trade_request_filled.amount_to_filled, 10_000);

    // CHARLIE: make sure the MM current trade is partially filled and correctly updated
    let trade_request_filled = Oracle::trades(trade_request_mm_id).unwrap();
    assert_eq!(trade_request_filled.status, SwapStatus::PartiallyFilled);
    // 100 TEMP
    assert_eq!(trade_request_filled.amount_from_filled, 10_000);
    // 5 tide
    assert_eq!(trade_request_filled.amount_to_filled, 5_000_000_000_000);

    // another partial filling who should close the trade
    assert_ok!(Oracle::confirm_swap(
      alice.clone(),
      trade_request_id,
      vec![
        // dave
        SwapConfirmation {
          request_id: trade_request_mm2_id,
          // 5 tide
          amount_to_receive: 5_000_000_000_000,
          // 100 TEMP
          amount_to_send: 10_000,
        },
      ],
    ));

    // BOB: make sure the CLIENT current trade is deleted
    assert!(Oracle::trades(trade_request_id).is_none());

    // cant send another trade confirmation as the request should be deleted
    // we do expect `InvalidRequestId`
    assert_noop!(
      Oracle::confirm_swap(alice.clone(), trade_request_id, vec![],),
      Error::<Test>::InvalidRequestId
    );

    // DAVE: make sure the MM current trade is partially filled and correctly updated
    let trade_request_filled = Oracle::trades(trade_request_mm2_id).unwrap();
    assert_eq!(trade_request_filled.status, SwapStatus::PartiallyFilled);
    // 100 TEMP
    assert_eq!(trade_request_filled.amount_from_filled, 10_000);
    // 5 tide
    assert_eq!(trade_request_filled.amount_to_filled, 5_000_000_000_000);

    // cancel our mm's swap to release the funds
    assert_ok!(Oracle::cancel_swap(alice.clone(), trade_request_mm_id,));
    assert_ok!(Oracle::cancel_swap(alice, trade_request_mm2_id,));

    // validate all balance
    assert_eq!(
      Adapter::balance(CurrencyId::Tide, &2u64),
      10_000_000_000_000
    );
    assert_eq!(
      Adapter::balance(CurrencyId::Wrapped(temp_asset_id), &2u64),
      20_000
    );
    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Tide, &2u64),
      Zero::zero()
    );

    assert_eq!(
      Adapter::balance(CurrencyId::Tide, &3u64),
      // swap + initial balance
      5_000_000_000_000 + 1_000_000_000_000
    );
    assert_eq!(
      Adapter::balance(CurrencyId::Wrapped(temp_asset_id), &3u64),
      1_000_000 - 10_000
    );
    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Wrapped(temp_asset_id), &3u64),
      Zero::zero()
    );

    assert_eq!(
      Adapter::balance(CurrencyId::Tide, &4u64),
      // swap + initial balance
      5_000_000_000_000 + 1_000_000_000_000
    );
    assert_eq!(
      Adapter::balance(CurrencyId::Wrapped(temp_asset_id), &4u64),
      1_000_000 - 10_000
    );
    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Wrapped(temp_asset_id), &4u64),
      Zero::zero()
    );
  });
}

#[test]
pub fn confirm_swap_simple_with_fees() {
  new_test_ext().execute_with(|| {
    let alice = Origin::signed(1u64);

    let temp_asset_id = 1;

    assert_ok!(Oracle::set_status(alice.clone(), true));
    assert!(Oracle::status());
    Fees::start_era();
    assert!(!Fees::active_era().is_none());

    // add 1 tide to alice & mm
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &1u64,
      1_000_000_000_000
    ));

    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &3u64,
      1_000_000_000_000
    ));

    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &4u64,
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
      &2u64,
      20_000_000_000_000
    ));

    // create TEMP asset
    assert_ok!(Assets::force_create(
      Origin::root(),
      temp_asset_id,
      1u64,
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
    assert_ok!(Assets::mint(alice.clone(), temp_asset_id, 3u64, 1_000_000));
    assert_ok!(Assets::mint(alice.clone(), temp_asset_id, 4u64, 1_000_000));

    // BOB: 10 TIDE for 200 TEMP (20 TEMP/TIDE)
    let (trade_request_id, trade_request) = Oracle::add_new_swap_in_queue(
      2u64,
      CurrencyId::Tide,
      10_000_000_000_000,
      CurrencyId::Wrapped(temp_asset_id),
      20_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
    )
    .unwrap();

    // CHARLIE (MM): 4000 TEMP FOR 200 TIDE
    let (trade_request_mm_id, trade_request_mm) = Oracle::add_new_swap_in_queue(
      3u64,
      CurrencyId::Wrapped(temp_asset_id),
      400_000,
      CurrencyId::Tide,
      200_000_000_000_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
    )
    .unwrap();

    // DAVE (MM): 100 TEMP for 5 TIDE
    let (trade_request_mm2_id, trade_request_mm2) = Oracle::add_new_swap_in_queue(
      4u64,
      CurrencyId::Wrapped(temp_asset_id),
      10_000,
      CurrencyId::Tide,
      5_000_000_000_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
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

    assert_eq!(trade_request.block_number, 0);
    assert_eq!(trade_request_mm.block_number, 0);

    // partial filling
    assert_ok!(Oracle::confirm_swap(
      alice,
      trade_request_id,
      vec![
        // charlie
        SwapConfirmation {
          request_id: trade_request_mm_id,
          // 5 tide
          amount_to_receive: 5_000_000_000_000,
          // 100 TEMP
          amount_to_send: 10_000,
        },
        // dave
        SwapConfirmation {
          request_id: trade_request_mm2_id,
          // 5 tide
          amount_to_receive: 5_000_000_000_000,
          // 100 TEMP
          amount_to_send: 10_000,
        },
      ],
    ));

    // BOB: make sure the CLIENT current trade is deleted
    assert!(Oracle::trades(trade_request_id).is_none());

    // CHARLIE: make sure the MM current trade is partially filled and correctly updated
    let trade_request_filled = Oracle::trades(trade_request_mm_id).unwrap();
    assert_eq!(trade_request_filled.status, SwapStatus::PartiallyFilled);
    // 100 TEMP
    assert_eq!(trade_request_filled.amount_from_filled, 10_000);
    // 5 tide
    assert_eq!(trade_request_filled.amount_to_filled, 5_000_000_000_000);

    // DAVE: make sure the MM current trade is totally filled (deleted)
    assert!(Oracle::trades(trade_request_mm2_id).is_none());

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

    // BOB Should have 9.8 tide remaining (started with 20), sent 10 tide and paid 2% fees
    assert_eq!(Adapter::balance(CurrencyId::Tide, &2u64), 9_800_000_000_000);

    // BOB Should have 200 TEMP
    assert_eq!(
      Adapter::balance(CurrencyId::Wrapped(temp_asset_id), &2u64),
      20_000
    );

    // make sure fees are registered on chain
    let bob_fee = Fees::account_fees(CurrencyId::Tide, 2u64);
    assert_eq!(bob_fee.fee, 200_000_000_000);
    assert_eq!(bob_fee.amount, 10_000_000_000_000);

    let charlie_fee = Fees::account_fees(CurrencyId::Wrapped(temp_asset_id), 3u64);
    assert_eq!(charlie_fee.fee, 200);
    assert_eq!(charlie_fee.amount, 10_000);

    let dave_fee = Fees::account_fees(CurrencyId::Wrapped(temp_asset_id), 4u64);
    assert_eq!(dave_fee.fee, 200);
    assert_eq!(dave_fee.amount, 10_000);
  });
}

#[test]
pub fn confirm_swap_ourself() {
  new_test_ext().execute_with(|| {
    let alice = Origin::signed(1u64);

    let temp_asset_id = 1;

    assert_ok!(Oracle::set_status(alice.clone(), true));
    assert!(Oracle::status());

    // set fee to 0%
    assert_ok!(Fees::set_fees_percentage(Origin::root(), Percent::zero()));

    // add 1 tide to alice & all MMs
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &1u64,
      1_000_000_000_000
    ));

    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &2u64,
      20_000_000_000_000
    ));

    // create TEMP asset
    assert_ok!(Assets::force_create(
      Origin::root(),
      temp_asset_id,
      1u64,
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
    assert_ok!(Assets::mint(alice.clone(), temp_asset_id, 2u64, 1_000_000));
    assert_eq!(
      Adapter::balance(CurrencyId::Wrapped(temp_asset_id), &2u64),
      1_000_000
    );

    // BOB: 10 TIDE for 200 TEMP (20 TEMP/TIDE)
    let (trade_request_id, trade_request) = Oracle::add_new_swap_in_queue(
      2u64,
      CurrencyId::Tide,
      10_000_000_000_000,
      CurrencyId::Wrapped(temp_asset_id),
      40_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Tide, &2u64),
      // we burned 1 tide on start so it should contain 1.2 tide now
      10_000_000_000_000
    );

    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Tide, &2u64, true),
      // minted 20_000_000_000_000 on genesis
      10_000_000_000_000
    );

    // CHARLIE (MM): 4000 TEMP FOR 200 TIDE
    let (trade_request_mm_id, trade_request_mm) = Oracle::add_new_swap_in_queue(
      2u64,
      CurrencyId::Wrapped(temp_asset_id),
      40_000,
      CurrencyId::Tide,
      10_000_000_000_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Wrapped(temp_asset_id), &2u64),
      40_000
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
      alice.clone(),
      trade_request_id,
      vec![
        // charlie
        SwapConfirmation {
          request_id: trade_request_mm_id,
          // 5 tide
          amount_to_receive: 10_000_000_000_000,
          amount_to_send: 40_000,
        },
      ],
    ));

    // BOB: make sure the CLIENT current trade is partially filled and correctly updated
    assert!(Oracle::trades(trade_request_id).is_none());
    assert!(Oracle::trades(trade_request_mm_id).is_none());

    // cant send another trade confirmation as the request should be deleted
    // we do expect `InvalidRequestId`
    assert_noop!(
      Oracle::confirm_swap(alice, trade_request_id, vec![],),
      Error::<Test>::InvalidRequestId
    );

    // validate all balance
    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Tide, &2u64, false),
      20_000_000_000_000
    );
    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Wrapped(temp_asset_id), &2u64, false),
      1_000_000
    );
    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Tide, &2u64),
      Zero::zero()
    );
    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Wrapped(temp_asset_id), &2u64),
      Zero::zero()
    );
  });
}
