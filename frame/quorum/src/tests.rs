use crate::{
  mock::{new_test_ext, Origin, Quorum, Test},
  Error,
};
use frame_support::{assert_noop, assert_ok};

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    assert!(!Quorum::status());
  });
}

#[test]
pub fn set_migration_operational_status_works() {
  new_test_ext().execute_with(|| {
    assert_ok!(Quorum::set_status(Origin::signed(1), true));
    assert_noop!(
      Quorum::set_status(Origin::signed(2), false),
      Error::<Test>::AccessDenied,
    );
    assert!(Quorum::status());
    assert_ok!(Quorum::set_status(Origin::signed(1), false));
    assert!(!Quorum::status());
  });
}
