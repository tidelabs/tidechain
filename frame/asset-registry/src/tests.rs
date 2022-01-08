use crate::mock::{new_test_ext, AssetRegistry};

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    // make sure we have TIDE + 2 custom currencies registered
    assert_eq!(
      AssetRegistry::get_assets()
        .expect("Unable to get results")
        .len(),
      3
    )
  });
}
