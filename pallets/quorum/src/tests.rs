use frame_support::{assert_noop, assert_ok};

use sp_runtime::traits::BadOrigin;

use crate::mock::{new_test_ext, Origin, Quorum};

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    assert!(!Quorum::is_quorum_enabled());
  });
}

#[test]
pub fn set_migration_operational_status_works() {
  new_test_ext().execute_with(|| {
    let non_sudo = 2u64;
    assert_ok!(Quorum::set_quorum_status(Origin::root(), true));
    assert_noop!(
      Quorum::set_quorum_status(Origin::signed(non_sudo), false),
      BadOrigin,
    );
    assert!(Quorum::is_quorum_enabled());
    assert_ok!(Quorum::set_quorum_status(Origin::root(), false));
    assert!(!Quorum::is_quorum_enabled());
  });
}
