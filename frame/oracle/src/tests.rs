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
    let bob_initial_balance: Balance = 20_000_000_000_000;

    assert_eq!(Fees::account_id(), 8246216774960574317);

    // add 1 tide to fees account to make sure account is valid
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &Fees::account_id(),
      1_000_000_000_000
    ));

    let temp_asset_id = 1;

    assert_ok!(Oracle::set_status(alice.clone(), true));
    assert!(Oracle::status());

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
      bob_initial_balance
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
    let charlie_initial_wrapped_balance: Balance = 1_000_000;
    assert_ok!(Assets::mint(
      alice.clone(),
      temp_asset_id,
      3u64,
      charlie_initial_wrapped_balance
    ));
    assert_eq!(
      Adapter::balance(CurrencyId::Wrapped(temp_asset_id), &3u64),
      charlie_initial_wrapped_balance
    );

    let dave_initial_wrapped_balance: Balance = 1_000_000;
    assert_ok!(Assets::mint(
      alice.clone(),
      temp_asset_id,
      4u64,
      dave_initial_wrapped_balance
    ));

    // BOB: 10 TIDE for 200 TEMP (20 TEMP/TIDE)
    let bob_initial_trade: Balance = 10_000_000_000_000;
    let bob_initial_trade_with_slippage =
      bob_initial_trade.saturating_add(Permill::from_percent(2) * bob_initial_trade);

    let (trade_request_id, trade_request) = Oracle::add_new_swap_in_queue(
      2u64,
      CurrencyId::Tide,
      bob_initial_trade,
      CurrencyId::Wrapped(temp_asset_id),
      20_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
      false,
      SwapType::Limit,
      // 2% slippage tolerance
      Permill::from_percent(2),
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Tide, &2u64),
      bob_initial_trade_with_slippage
        // add 0.2% fee
        .saturating_add(FeeAmount::get() * bob_initial_trade_with_slippage)
    );

    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Tide, &2u64, true),
      bob_initial_balance
        // reduce 2% slippage
        .saturating_sub(bob_initial_trade_with_slippage)
        // reduce 0.2% network fee
        .saturating_sub(FeeAmount::get() * bob_initial_trade_with_slippage)
    );

    // CHARLIE (MM): 4000 TEMP FOR 200 TIDE
    let charlie_initial_trade: Balance = 400_000;
    let charlie_initial_trade_with_slippage =
      charlie_initial_trade.saturating_add(Permill::from_percent(4) * charlie_initial_trade);

    let (trade_request_mm_id, trade_request_mm) = Oracle::add_new_swap_in_queue(
      3u64,
      CurrencyId::Wrapped(temp_asset_id),
      charlie_initial_trade,
      CurrencyId::Tide,
      200_000_000_000_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1,
      ],
      true,
      SwapType::Limit,
      // 4% slippage tolerance
      Permill::from_percent(4),
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Wrapped(temp_asset_id), &3u64),
      charlie_initial_trade_with_slippage
        // add 0.1% fee
        .saturating_add(MarketMakerFeeAmount::get() * charlie_initial_trade_with_slippage)
    );

    let last_charlie_balance = charlie_initial_wrapped_balance
      .saturating_sub(charlie_initial_trade_with_slippage)
      .saturating_sub(MarketMakerFeeAmount::get() * charlie_initial_trade_with_slippage);

    assert_eq!(
      Adapter::balance(CurrencyId::Wrapped(temp_asset_id), &3u64),
      last_charlie_balance
    );

    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Wrapped(temp_asset_id), &3u64, true),
      charlie_initial_wrapped_balance
        // keep-alive token
        .saturating_sub(1_u128)
        // slippage
        .saturating_sub(charlie_initial_trade_with_slippage)
        // fees
        .saturating_sub(MarketMakerFeeAmount::get() * charlie_initial_trade_with_slippage)
    );

    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Wrapped(temp_asset_id), &3u64, false),
      // minted 1_000_000 on genesis (no keep-alive)
      charlie_initial_wrapped_balance
        // slippage
        .saturating_sub(charlie_initial_trade_with_slippage)
        // fees
        .saturating_sub(MarketMakerFeeAmount::get() * charlie_initial_trade_with_slippage)
    );

    // DAVE (MM): 8000 TEMP for 400 TIDE
    let dave_initial_trade: Balance = 800_000;
    let dave_initial_trade_with_slippage =
      dave_initial_trade.saturating_add(Permill::from_percent(5) * dave_initial_trade);

    let (trade_request_mm2_id, trade_request_mm2) = Oracle::add_new_swap_in_queue(
      4u64,
      CurrencyId::Wrapped(temp_asset_id),
      800_000,
      CurrencyId::Tide,
      400_000_000_000_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 2,
      ],
      true,
      SwapType::Limit,
      // 5% slippage tolerance
      Permill::from_percent(5),
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Wrapped(temp_asset_id), &4u64),
      dave_initial_trade_with_slippage
        // add 0.1% fee
        .saturating_add(MarketMakerFeeAmount::get() * dave_initial_trade_with_slippage)
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
      Adapter::balance(CurrencyId::Tide, &2u64),
      20_000_000_000_000
    );

    let partial_filling_amount_charlie: Balance = 10_000;
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
          amount_to_send: partial_filling_amount_charlie,
        },
      ],
    ));

    assert_eq!(
      Adapter::balance(CurrencyId::Tide, &2u64),
      bob_initial_balance
        // reduce 2% slippage
        .saturating_sub(5_000_000_000_000)
        // reduce 0.2% network fee
        .saturating_sub(FeeAmount::get() * 5_000_000_000_000)
    );

    // swap confirmation for bob (user)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_id,
      status: SwapStatus::PartiallyFilled,
      account_id: 2u64,
      currency_from: CurrencyId::Tide,
      currency_amount_from: 5_000_000_000_000,
      currency_to: CurrencyId::Wrapped(temp_asset_id),
      currency_amount_to: partial_filling_amount_charlie,
      initial_extrinsic_hash: [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
    }));

    // swap confirmation for charlie (mm)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_mm_id,
      status: SwapStatus::PartiallyFilled,
      account_id: 3u64,
      currency_from: CurrencyId::Wrapped(temp_asset_id),
      currency_amount_from: partial_filling_amount_charlie,
      currency_to: CurrencyId::Tide,
      currency_amount_to: 5_000_000_000_000,
      initial_extrinsic_hash: [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1,
      ],
    }));

    // BOB: make sure the CLIENT current trade is partially filled and correctly updated
    let trade_request_filled = Oracle::swaps(trade_request_id).unwrap();
    assert_eq!(trade_request_filled.status, SwapStatus::PartiallyFilled);

    let trade_request_account = Oracle::account_swaps(2u64).unwrap();
    assert_eq!(
      trade_request_account
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_id),
      Some(&(trade_request_id, SwapStatus::PartiallyFilled))
    );

    // 5 tide
    assert_eq!(trade_request_filled.amount_from_filled, 5_000_000_000_000);
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
          amount_to_send: partial_filling_amount_charlie,
        },
      ],
    ));

    assert_eq!(
      Adapter::balance(CurrencyId::Tide, &2u64),
      bob_initial_balance
        .saturating_sub(10_000_000_000_000)
        // reduce 0.2% network fee
        .saturating_sub(FeeAmount::get() * 10_000_000_000_000)
    );

    // swap confirmation for bob (user)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_id,
      status: SwapStatus::Completed,
      account_id: 2u64,
      currency_from: CurrencyId::Tide,
      currency_amount_from: 5_000_000_000_000,
      currency_to: CurrencyId::Wrapped(temp_asset_id),
      currency_amount_to: partial_filling_amount_charlie,
      initial_extrinsic_hash: [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
    }));

    // swap confirmation for dave (second mm)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_mm2_id,
      status: SwapStatus::PartiallyFilled,
      account_id: 4u64,
      currency_from: CurrencyId::Wrapped(temp_asset_id),
      currency_amount_from: partial_filling_amount_charlie,
      currency_to: CurrencyId::Tide,
      currency_amount_to: 5_000_000_000_000,
      initial_extrinsic_hash: [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 2,
      ],
    }));

    // BOB: make sure the CLIENT current trade is deleted
    assert!(Oracle::swaps(trade_request_id).is_none());
    let trade_request_account = Oracle::account_swaps(2u64).unwrap();
    assert_eq!(
      trade_request_account
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_id),
      None
    );

    // cant send another trade confirmation as the request should be deleted
    // we do expect `InvalidRequestId`
    assert_noop!(
      Oracle::confirm_swap(alice.clone(), trade_request_id, vec![],),
      Error::<Test>::InvalidRequestId
    );

    // DAVE: make sure the MM current trade is partially filled and correctly updated
    let trade_request_filled = Oracle::swaps(trade_request_mm2_id).unwrap();
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
      bob_initial_balance
        .saturating_sub(10_000_000_000_000)
        // reduce 0.2% network fee
        .saturating_sub(FeeAmount::get() * 10_000_000_000_000)
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
      Adapter::reducible_balance(CurrencyId::Wrapped(temp_asset_id), &3u64, false),
      charlie_initial_wrapped_balance
        .saturating_sub(partial_filling_amount_charlie)
        .saturating_sub(MarketMakerFeeAmount::get() * partial_filling_amount_charlie)
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
      dave_initial_wrapped_balance
        .saturating_sub(partial_filling_amount_charlie)
        .saturating_sub(MarketMakerFeeAmount::get() * partial_filling_amount_charlie)
    );

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Wrapped(temp_asset_id), &4u64),
      Zero::zero()
    );
  });
}

#[test]
pub fn confirm_swap_with_fees() {
  new_test_ext().execute_with(|| {
    let alice = Origin::signed(1u64);

    let temp_asset_id = 1;

    assert_ok!(Oracle::set_status(alice.clone(), true));
    assert!(Oracle::status());
    Fees::start_era();
    assert!(!Fees::active_era().is_none());
    let current_era = Fees::active_era().unwrap().index;

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
      false,
      SwapType::Limit,
      // 2% slippage tolerance
      Permill::from_percent(2),
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
        0, 1,
      ],
      true,
      SwapType::Limit,
      // 5% slippage tolerance
      Permill::from_percent(5),
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
        0, 2,
      ],
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
      Oracle::account_swaps(2u64)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_id),
      Some(&(trade_request_id, SwapStatus::Pending))
    );

    assert_eq!(
      Oracle::account_swaps(3u64)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_mm_id),
      Some(&(trade_request_mm_id, SwapStatus::Pending))
    );

    assert_eq!(
      Oracle::account_swaps(4u64)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_mm2_id),
      Some(&(trade_request_mm2_id, SwapStatus::Pending))
    );

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

    // swap confirmation for bob (user)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_id,
      status: SwapStatus::Completed,
      account_id: 2u64,
      currency_from: CurrencyId::Tide,
      currency_amount_from: 10_000_000_000_000,
      currency_to: CurrencyId::Wrapped(temp_asset_id),
      currency_amount_to: 20_000,
      initial_extrinsic_hash: [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
    }));

    // swap confirmation for charlie (mm1)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_mm_id,
      status: SwapStatus::PartiallyFilled,
      account_id: 3u64,
      currency_from: CurrencyId::Wrapped(temp_asset_id),
      currency_amount_from: 10_000,
      currency_to: CurrencyId::Tide,
      currency_amount_to: 5_000_000_000_000,
      initial_extrinsic_hash: [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1,
      ],
    }));

    // swap confirmation for dave (mm2)
    // the trade should be closed, because amount_from of the request is filled
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_mm2_id,
      status: SwapStatus::Completed,
      account_id: 4u64,
      currency_from: CurrencyId::Wrapped(temp_asset_id),
      currency_amount_from: 10_000,
      currency_to: CurrencyId::Tide,
      currency_amount_to: 5_000_000_000_000,
      initial_extrinsic_hash: [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 2,
      ],
    }));

    // BOB: make sure the CLIENT current trade is deleted
    assert!(Oracle::swaps(trade_request_id).is_none());
    assert_eq!(
      Oracle::account_swaps(2u64)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_id),
      None
    );

    // CHARLIE: make sure the MM current trade is partially filled and correctly updated
    assert_eq!(
      Oracle::account_swaps(3u64)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_mm_id),
      Some(&(trade_request_mm_id, SwapStatus::PartiallyFilled))
    );

    let trade_request_filled = Oracle::swaps(trade_request_mm_id).unwrap();
    assert_eq!(trade_request_filled.status, SwapStatus::PartiallyFilled);
    // 100 TEMP
    assert_eq!(trade_request_filled.amount_from_filled, 10_000);
    // 5 tide
    assert_eq!(trade_request_filled.amount_to_filled, 5_000_000_000_000);

    // DAVE: make sure the MM current trade is totally filled (deleted)
    assert!(Oracle::swaps(trade_request_mm2_id).is_none());
    assert_eq!(
      Oracle::account_swaps(4u64)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_mm2_id),
      None
    );

    // make sure all balances match
    assert_eq!(
      Adapter::balance(CurrencyId::Tide, &Fees::account_id()),
      // we burned 1 tide on start so it should contain 1.2 tide now
      1_200_000_000_000
    );

    assert_eq!(
      Adapter::balance(CurrencyId::Wrapped(temp_asset_id), &Fees::account_id()),
      200
    );

    // BOB Should have 9.8 tide remaining (started with 20), sent 10 tide and paid 2% fees
    assert_eq!(Adapter::balance(CurrencyId::Tide, &2u64), 9_800_000_000_000);

    // BOB Should have 200 TEMP
    assert_eq!(
      Adapter::balance(CurrencyId::Wrapped(temp_asset_id), &2u64),
      20_000
    );

    // make sure fees are registered on chain
    let bob_fee = Fees::account_fees(current_era, 2u64);
    assert_eq!(bob_fee.first().unwrap().1.fee, 200_000_000_000);
    assert_eq!(bob_fee.first().unwrap().1.amount, 10_000_000_000_000);

    let charlie_fee = Fees::account_fees(current_era, 3u64);
    assert_eq!(charlie_fee.first().unwrap().1.fee, 100);
    assert_eq!(charlie_fee.first().unwrap().1.amount, 10_000);

    let dave_fee = Fees::account_fees(current_era, 4u64);
    assert_eq!(dave_fee.first().unwrap().1.fee, 100);
    assert_eq!(dave_fee.first().unwrap().1.amount, 10_000);
  });
}

#[test]
pub fn confirm_swap_ourself() {
  new_test_ext().execute_with(|| {
    let alice = Origin::signed(1u64);

    let temp_asset_id = 1;

    let bob_initial_balance: Balance = 20_000_000_000_000;
    let bob_initial_wrapped_balance: Balance = 1_000_000;

    assert_ok!(Oracle::set_status(alice.clone(), true));
    assert!(Oracle::status());

    // add 1 tide to alice & all MMs
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &1u64,
      1_000_000_000_000
    ));

    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &2u64,
      bob_initial_balance
    ));

    // create TEMP asset
    assert_ok!(Assets::force_create(
      Origin::root(),
      temp_asset_id,
      1u64,
      true,
      1
    ));

    assert_eq!(Fees::account_id(), 8246216774960574317);

    // add 1 tide to fees account to make sure account is valid
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &Fees::account_id(),
      1_000_000_000_000
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
      2u64,
      bob_initial_wrapped_balance
    ));
    assert_eq!(
      Adapter::balance(CurrencyId::Wrapped(temp_asset_id), &2u64),
      bob_initial_wrapped_balance
    );

    // BOB: 10 TIDE for 200 TEMP (20 TEMP/TIDE)
    let bob_initial_trade: Balance = 10_000_000_000_000;
    let bob_initial_trade_with_slippage =
      bob_initial_trade.saturating_add(Permill::from_percent(2) * bob_initial_trade);

    let (trade_request_id, trade_request) = Oracle::add_new_swap_in_queue(
      2u64,
      CurrencyId::Tide,
      bob_initial_trade,
      CurrencyId::Wrapped(temp_asset_id),
      40_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
      false,
      SwapType::Limit,
      // 2% slippage tolerance
      Permill::from_percent(2),
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Tide, &2u64),
      bob_initial_trade_with_slippage
        // add 0.2% fee
        .saturating_add(FeeAmount::get() * bob_initial_trade_with_slippage)
    );

    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Tide, &2u64, true),
      bob_initial_balance
        // reduce 2% slippage
        .saturating_sub(bob_initial_trade_with_slippage)
        // reduce 0.2% network fee
        .saturating_sub(FeeAmount::get() * bob_initial_trade_with_slippage)
    );

    let bob_initial_trade: Balance = 40_000;
    let bob_initial_trade_with_slippage =
      bob_initial_trade.saturating_add(Permill::from_percent(5) * bob_initial_trade);

    let (trade_request_mm_id, trade_request_mm) = Oracle::add_new_swap_in_queue(
      2u64,
      CurrencyId::Wrapped(temp_asset_id),
      bob_initial_trade,
      CurrencyId::Tide,
      10_000_000_000_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
      true,
      SwapType::Limit,
      // 5% slippage tolerance
      Permill::from_percent(5),
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Wrapped(temp_asset_id), &2u64),
      bob_initial_trade_with_slippage
        // add 0.1% fee
        .saturating_add(MarketMakerFeeAmount::get() * bob_initial_trade_with_slippage)
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
          amount_to_receive: 10_000_000_000_000,
          amount_to_send: 40_000,
        },
      ],
    ));

    // BOB: make sure the CLIENT current trade is partially filled and correctly updated
    assert!(Oracle::swaps(trade_request_id).is_none());
    assert!(Oracle::swaps(trade_request_mm_id).is_none());

    // cant send another trade confirmation as the request should be deleted
    // we do expect `InvalidRequestId`
    assert_noop!(
      Oracle::confirm_swap(alice, trade_request_id, vec![],),
      Error::<Test>::InvalidRequestId
    );

    // validate all balance
    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Tide, &2u64, false),
      // we should refund the extra fees paid on the slippage value
      bob_initial_balance.saturating_sub(FeeAmount::get() * 10_000_000_000_000)
    );

    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Wrapped(temp_asset_id), &2u64, false),
      bob_initial_wrapped_balance.saturating_sub(MarketMakerFeeAmount::get() * 40_000)
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

#[test]
pub fn test_slippage() {
  new_test_ext().execute_with(|| {
    let alice = Origin::signed(1u64);

    let temp_asset_id = 1;

    let bob_initial_balance: Balance = 20_000_000_000_000;
    let bob_initial_wrapped_balance: Balance = 1_000_000;

    assert_ok!(Oracle::set_status(alice.clone(), true));
    assert!(Oracle::status());

    // add 1 tide to alice & all MMs
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &1u64,
      1_000_000_000_000
    ));

    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &2u64,
      bob_initial_balance
    ));

    // create TEMP asset
    assert_ok!(Assets::force_create(
      Origin::root(),
      temp_asset_id,
      1u64,
      true,
      1
    ));

    assert_eq!(Fees::account_id(), 8246216774960574317);

    // add 1 tide to fees account to make sure account is valid
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &Fees::account_id(),
      1_000_000_000_000
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
      2u64,
      bob_initial_wrapped_balance
    ));
    assert_eq!(
      Adapter::balance(CurrencyId::Wrapped(temp_asset_id), &2u64),
      bob_initial_wrapped_balance
    );

    // BOB: 10 TIDE for 200 TEMP (20 TEMP/TIDE)
    let bob_initial_trade: Balance = 10_000_000_000_000;
    let bob_initial_trade_with_slippage =
      bob_initial_trade.saturating_add(Permill::from_percent(2) * bob_initial_trade);

    let (trade_request_id, trade_request) = Oracle::add_new_swap_in_queue(
      2u64,
      CurrencyId::Tide,
      bob_initial_trade,
      CurrencyId::Wrapped(temp_asset_id),
      40_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
      false,
      SwapType::Limit,
      // 2% slippage tolerance
      Permill::from_percent(2),
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Tide, &2u64),
      bob_initial_trade_with_slippage
        // add 0.2% fee
        .saturating_add(FeeAmount::get() * bob_initial_trade_with_slippage)
    );

    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Tide, &2u64, true),
      bob_initial_balance
        // reduce 2% slippage
        .saturating_sub(bob_initial_trade_with_slippage)
        // reduce 0.2% network fee
        .saturating_sub(FeeAmount::get() * bob_initial_trade_with_slippage)
    );

    let bob_initial_trade: Balance = 80_000;
    let bob_initial_trade_with_slippage =
      bob_initial_trade.saturating_add(Permill::from_percent(5) * bob_initial_trade);

    let (trade_request_mm_id, trade_request_mm) = Oracle::add_new_swap_in_queue(
      2u64,
      CurrencyId::Wrapped(temp_asset_id),
      bob_initial_trade,
      CurrencyId::Tide,
      20_000_000_000_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
      true,
      SwapType::Market,
      // 5% slippage tolerance
      Permill::from_percent(5),
    )
    .unwrap();

    let (trade_request_mm_id2, _) = Oracle::add_new_swap_in_queue(
      2u64,
      CurrencyId::Wrapped(temp_asset_id),
      bob_initial_trade,
      CurrencyId::Tide,
      10_000_000_000_000,
      0,
      [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
      ],
      true,
      SwapType::Market,
      // 5% slippage tolerance
      Permill::from_percent(5),
    )
    .unwrap();

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Wrapped(temp_asset_id), &2u64),
      bob_initial_trade_with_slippage
        // add 0.1% fee
        .saturating_add(bob_initial_trade_with_slippage)
        .saturating_add(MarketMakerFeeAmount::get() * bob_initial_trade_with_slippage)
        .saturating_add(MarketMakerFeeAmount::get() * bob_initial_trade_with_slippage)
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
        alice.clone(),
        trade_request_id,
        vec![SwapConfirmation {
          request_id: trade_request_mm_id,
          amount_to_receive: 10_210_000_000_000,
          amount_to_send: 40_000,
        },],
      ),
      Error::<Test>::Overflow
    );

    assert_noop!(
      Oracle::confirm_swap(
        alice.clone(),
        trade_request_id,
        vec![SwapConfirmation {
          request_id: trade_request_mm_id2,
          amount_to_receive: 10_000_000_000_000,
          amount_to_send: 40_000,
        },],
      ),
      Error::<Test>::MarketMakerOverflow
    );

    // partial filling
    assert_ok!(Oracle::confirm_swap(
      alice.clone(),
      trade_request_id,
      vec![SwapConfirmation {
        request_id: trade_request_mm_id,
        amount_to_receive: 10_200_000_000_000,
        amount_to_send: 40_000,
      },],
    ));

    assert!(Oracle::swaps(trade_request_id).is_none());
    assert!(Oracle::swaps(trade_request_mm_id).is_none());

    // cant send another trade confirmation as the request should be deleted
    // we do expect `InvalidRequestId`
    assert_noop!(
      Oracle::confirm_swap(alice, trade_request_id, vec![],),
      Error::<Test>::InvalidRequestId
    );
  });
}
