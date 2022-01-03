use crate::mock::{new_test_ext, AssetRegistry};
use tidefi_primitives::{CurrencyId, CurrencyMetadata};

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    assert_eq!(
      AssetRegistry::get_assets(),
      Ok(vec![
        (
          CurrencyId::Tide,
          CurrencyMetadata {
            name: "Tide".into(),
            symbol: "TIDE".into(),
            decimals: 12,
            is_frozen: false
          }
        ),
        (
          CurrencyId::Wrapped(1),
          CurrencyMetadata {
            name: "Test".into(),
            symbol: "TST".into(),
            decimals: 6,
            is_frozen: false
          }
        )
      ])
    );
  });
}
