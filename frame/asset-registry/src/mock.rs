// Copyright 2021-2022 Semantic Network Ltd.
// This file is part of Tidechain.

// Tidechain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tidechain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tidechain.  If not, see <http://www.gnu.org/licenses/>.

use crate::pallet as pallet_asset_registry;
use frame_support::{
  traits::{ConstU128, EnsureOrigin},
  PalletId,
};
use frame_system as system;
use frame_utils::construct_mock_runtime;
use sp_runtime::traits::AccountIdConversion;
use system::RawOrigin;
use tidefi_primitives::CurrencyId;

construct_mock_runtime!({
  Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
  AssetRegistry: pallet_asset_registry::{Pallet, Call, Config<T>, Storage, Event<T>},
}, {
  pub const AssetRegistryPalletId: PalletId = PalletId(*b"wrpr*art");
  pub const AssetDeposit: u64 = 1;
  pub const ApprovalDeposit: u64 = 1;
  pub const StringLimit: u32 = 50;
  pub const MetadataDepositBase: u64 = 1;
  pub const MetadataDepositPerByte: u64 = 1;
});

pub struct EnsureRootOrAssetRegistry;
impl EnsureOrigin<Origin> for EnsureRootOrAssetRegistry {
  type Success = AccountId;

  fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
    Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
      RawOrigin::Root => Ok(AssetRegistryPalletId::get().into_account_truncating()),
      RawOrigin::Signed(caller) => {
        let asset_registry_account: AccountId =
          AssetRegistryPalletId::get().into_account_truncating();
        // Allow call from asset registry pallet ID account
        if caller == asset_registry_account
        // Allow call from asset registry owner
        || caller == AssetRegistry::account_id().expect("Unable to get asset registry account id")
        {
          Ok(caller)
        } else {
          Err(Origin::from(Some(caller)))
        }
      }
      r => Err(Origin::from(r)),
    })
  }

  #[cfg(feature = "runtime-benchmarks")]
  fn successful_origin() -> Origin {
    Origin::from(RawOrigin::Signed(Default::default()))
  }
}

impl pallet_assets::Config for Test {
  type Event = Event;
  type Balance = u128;
  type AssetId = u32;
  type Currency = Balances;
  type ForceOrigin = EnsureRootOrAssetRegistry;
  type AssetDeposit = AssetDeposit;
  type MetadataDepositBase = MetadataDepositBase;
  type MetadataDepositPerByte = MetadataDepositPerByte;
  type ApprovalDeposit = ApprovalDeposit;
  type StringLimit = StringLimit;
  type Freezer = ();
  type Extra = ();
  type WeightInfo = ();
  type AssetAccountDeposit = ConstU128<0>;
}

impl pallet_asset_registry::Config for Test {
  type Event = Event;
  type WeightInfo = crate::weights::SubstrateWeight<Test>;
  type AssetRegistryPalletId = AssetRegistryPalletId;
  // Wrapped currency
  type CurrencyTidefi = Adapter<AccountId>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
  let mut t = system::GenesisConfig::default()
    .build_storage::<Test>()
    .unwrap();

  pallet_balances::GenesisConfig::<Test>::default()
    .assimilate_storage(&mut t)
    .unwrap();

  pallet_asset_registry::GenesisConfig::<Test> {
    assets: vec![
      (
        CurrencyId::Wrapped(100),
        "Bitcoin".into(),
        "BTC".into(),
        8,
        Vec::new(),
      ),
      (
        CurrencyId::Wrapped(1000),
        "Ethereum".into(),
        "ETH".into(),
        18,
        Vec::new(),
      ),
    ],
    account: 0_u64.into(),
  }
  .assimilate_storage(&mut t)
  .unwrap();

  t.into()
}
