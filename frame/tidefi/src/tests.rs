use crate::{
  mock::{new_test_ext, Adapter, Assets, Event as MockEvent, Origin, System, Tidefi},
  pallet::*,
};
use frame_support::{assert_ok, traits::fungibles::Mutate};
use sp_runtime::Permill;
use std::str::FromStr;
use tidefi_primitives::{CurrencyId, Hash, SwapType};

#[test]
pub fn request_swap_event() {
  new_test_ext().execute_with(|| {
    let alice = Origin::signed(1u64);
    let temp_asset_id = 1;

    // add 1 tide to alice & all MMs
    assert_ok!(Adapter::mint_into(
      CurrencyId::Tide,
      &1u64,
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

    // mint TEMP funds to bob
    assert_ok!(Assets::mint(alice, temp_asset_id, 2u64, 1_000_000));

    // Submit request
    assert_ok!(Tidefi::swap(
      Origin::signed(2u64),
      CurrencyId::Tide,
      10_000_000_000_000,
      CurrencyId::Wrapped(temp_asset_id),
      20_000,
      SwapType::Limit,
      None
    ));

    // swap confirmation for bob (user)
    System::assert_has_event(MockEvent::Tidefi(Event::Swap {
      request_id: Hash::from_str(
        "0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e",
      )
      .unwrap_or_default(),
      account: 2u64,
      currency_id_from: CurrencyId::Tide,
      amount_from: 10_000_000_000_000,
      currency_id_to: CurrencyId::Wrapped(temp_asset_id),
      amount_to: 20_000,
      extrinsic_hash: [
        14, 87, 81, 192, 38, 229, 67, 178, 232, 171, 46, 176, 96, 153, 218, 161, 209, 229, 223, 71,
        119, 143, 119, 135, 250, 171, 69, 205, 241, 47, 227, 168,
      ],
      slippage_tolerance: Permill::zero(),
      swap_type: SwapType::Limit,
      is_market_maker: false,
    }));
  })
}
