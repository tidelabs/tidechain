use crate::mock::{new_test_ext, AccountId, Fees};
use frame_support::assert_ok;
use tidefi_primitives::{pallet::FeesExt, CurrencyId};

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    assert!(Fees::active_era().is_some());
  });
}

#[test]
pub fn calculate_trading_fees() {
  new_test_ext().execute_with(|| {
    // 100 tide @ 2% should cost 2 TIDEs
    let calculated_fee = Fees::calculate_swap_fees(CurrencyId::Tide, 100_000_000_000_000, false);
    assert_eq!(calculated_fee.amount, 100_000_000_000_000);
    assert_eq!(calculated_fee.fee, 2_000_000_000_000);

    let calculated_fee = Fees::calculate_swap_fees(CurrencyId::Tide, 100_000_000_000_000, true);
    assert_eq!(calculated_fee.amount, 100_000_000_000_000);
    assert_eq!(calculated_fee.fee, 1_000_000_000_000);
  });
}

#[test]
pub fn register_swap_fees() {
  new_test_ext().execute_with(|| {
    let current_era = Fees::active_era().unwrap().index;
    Fees::start_era();
    let new_current_era = Fees::active_era().unwrap().index;
    assert_eq!(current_era + 1, new_current_era);

    // 100 tide @ 2% should cost 2 TIDEs
    let calculated_fee =
      Fees::register_swap_fees(3u64.into(), CurrencyId::Tide, 100_000_000_000_000, false).unwrap();
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
      CurrencyId::Tide,
      100_000_000_000_000,
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
