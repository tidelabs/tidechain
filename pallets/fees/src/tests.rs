use crate::mock::{new_test_ext, AccountId, Fees};
use sp_runtime::Percent;

use tidefi_primitives::{pallet::FeesExt, CurrencyId};

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    assert_eq!(Fees::fee_percentage(), Percent::from_parts(2));
    assert_eq!(Fees::distribution_percentage(), Percent::from_parts(20));
    assert!(Fees::active_era().is_none());
  });
}

#[test]
pub fn calculate_trading_fees() {
  new_test_ext().execute_with(|| {
    // 100 tide @ 2% should cost 2 TIDEs
    let calculated_fee = Fees::calculate_trading_fees(CurrencyId::Tide, 100_000_000_000_000);
    assert_eq!(calculated_fee.amount, 100_000_000_000_000);
    assert_eq!(calculated_fee.fee, 2_000_000_000_000);
  });
}

#[test]
pub fn register_trading_fees() {
  new_test_ext().execute_with(|| {
    Fees::start_era();
    assert_eq!(Fees::active_era().is_none(), false);

    // 100 tide @ 2% should cost 2 TIDEs
    let calculated_fee =
      Fees::register_trading_fees(3u64.into(), CurrencyId::Tide, 100_000_000_000_000);
    assert_eq!(calculated_fee.amount, 100_000_000_000_000);
    assert_eq!(calculated_fee.fee, 2_000_000_000_000);

    // make sure everything was registered
    let registered_fee = Fees::account_fees(CurrencyId::Tide, AccountId(3u64));
    assert_eq!(registered_fee.amount, 100_000_000_000_000);
    assert_eq!(registered_fee.fee, 2_000_000_000_000);

    // make sure it increment the value
    Fees::register_trading_fees(3u64.into(), CurrencyId::Tide, 100_000_000_000_000);
    let registered_fee = Fees::account_fees(CurrencyId::Tide, AccountId(3u64));
    assert_eq!(registered_fee.amount, 200_000_000_000_000);
    assert_eq!(registered_fee.fee, 4_000_000_000_000);
  });
}
