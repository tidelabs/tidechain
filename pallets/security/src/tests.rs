use frame_support::{assert_noop, assert_ok};

use sp_runtime::traits::BadOrigin;

use crate::mock::new_test_ext;

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {});
}

#[test]
pub fn set_migration_operational_status_works() {
  new_test_ext().execute_with(|| {});
}
