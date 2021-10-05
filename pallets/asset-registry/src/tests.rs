



use crate::mock::new_test_ext;

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {});
}

#[test]
pub fn set_migration_operational_status_works() {
  new_test_ext().execute_with(|| {});
}
