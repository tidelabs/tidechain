use frame_support::{assert_noop, assert_ok};

use sp_runtime::traits::BadOrigin;

use crate::mock::{new_test_ext, Origin, Quorum};

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    assert!(!Quorum::status());
  });
}

#[test]
pub fn set_migration_operational_status_works() {
  new_test_ext().execute_with(|| {
    let non_sudo = 2u64;
    assert_ok!(Quorum::set_status(Origin::root(), true));
    assert_noop!(
      Quorum::set_status(Origin::signed(non_sudo.into()), false),
      BadOrigin,
    );
    assert!(Quorum::status());
    assert_ok!(Quorum::set_status(Origin::root(), false));
    assert!(!Quorum::status());
  });
}
