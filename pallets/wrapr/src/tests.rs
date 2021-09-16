use frame_support::{assert_noop, assert_ok};

use sp_runtime::traits::BadOrigin;

use crate::mock::{new_test_ext, Origin, TideWrapr};

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    assert!(!TideWrapr::is_quorum_enabled());
    assert_eq!(TideWrapr::highest_token(), 8);
  });
}

#[test]
pub fn set_migration_operational_status_works() {
  new_test_ext().execute_with(|| {
    let non_sudo = 2u64;
    assert_ok!(TideWrapr::set_quorum_status(Origin::root(), true));
    assert_noop!(
      TideWrapr::set_quorum_status(Origin::signed(non_sudo), false),
      BadOrigin,
    );
    assert!(TideWrapr::is_quorum_enabled());
    assert_ok!(TideWrapr::set_quorum_status(Origin::root(), false));
    assert!(!TideWrapr::is_quorum_enabled());
  });
}
