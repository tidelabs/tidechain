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

use crate::{
  constants::currency::{deposit, Adapter, CENTS, DOLLARS},
  types::{AccountId, AssetId, Balance, BlockNumber},
  AssetRegistryPalletId, Balances, CouncilCollectiveInstance, Event, FeesPalletId, OraclePalletId,
  Origin, QuorumPalletId, Runtime, Timestamp, WraprAssetRegistry, WraprFees, WraprOracle,
  WraprQuorum, WraprSecurity, WraprStakePalletId,
};

use frame_support::{
  parameter_types,
  traits::{ConstU128, EnsureOneOf, EnsureOrigin},
};
use frame_system::{EnsureRoot, RawOrigin};
use sp_core::u32_trait::{_2, _3};
use sp_runtime::traits::AccountIdConversion;

parameter_types! {
  pub const ApprovalDeposit: Balance = 10 * CENTS;
  pub const AssetDeposit: Balance = DOLLARS;
  pub const AssetsStringLimit: u32 = 50;
  /// Key = 32 bytes, Value = 36 bytes (32+1+1+1+1)
  // https://github.com/paritytech/substrate/blob/069917b/frame/assets/src/lib.rs#L257L271
  pub const MetadataDepositBase: Balance = deposit(1, 68);
  pub const MetadataDepositPerByte: Balance = deposit(0, 1);

  // FIXME: Should be better than that as we have multiple basis
  pub const PeriodBasis: BlockNumber = 1000u32;
}

pub struct EnsureRootOrAssetRegistry;
impl EnsureOrigin<Origin> for EnsureRootOrAssetRegistry {
  type Success = AccountId;

  fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
    Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
      RawOrigin::Root => Ok(AssetRegistryPalletId::get().into_account()),
      RawOrigin::Signed(caller) => {
        // Allow call from asset registry pallet ID account
        if caller == AssetRegistryPalletId::get().into_account()
         // Allow call from asset registry owner
         || Some(caller.clone()) == WraprAssetRegistry::account_id()
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
    Origin::from(RawOrigin::Signed(
      AssetRegistryPalletId::get().into_account(),
    ))
  }
}

impl pallet_assets::Config for Runtime {
  type Event = Event;
  type Balance = Balance;
  type AssetId = AssetId;
  type Currency = Balances;
  type ForceOrigin = EnsureRootOrAssetRegistry;
  type AssetDeposit = AssetDeposit;
  type MetadataDepositBase = MetadataDepositBase;
  type MetadataDepositPerByte = MetadataDepositPerByte;
  type ApprovalDeposit = ApprovalDeposit;
  type StringLimit = AssetsStringLimit;
  type Freezer = ();
  // The amount of funds that must be reserved for a non-provider asset account to be
  // maintained.
  type AssetAccountDeposit = ConstU128<0>;
  // FIXME: Use local weight
  type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
  type Extra = ();
}

impl pallet_wrapr::Config for Runtime {
  type Event = Event;
  type Quorum = WraprQuorum;
  type Oracle = WraprOracle;
  // FIXME: Use local weight
  type WeightInfo = pallet_wrapr::weights::SubstrateWeight<Runtime>;
  // Wrapped currency
  type CurrencyWrapr = Adapter<AccountId>;
  // Asset registry
  type AssetRegistry = WraprAssetRegistry;
}

impl pallet_wrapr_stake::Config for Runtime {
  type Event = Event;
  type StakePalletId = WraprStakePalletId;
  // FIXME: Use local weight
  type WeightInfo = pallet_wrapr_stake::weights::SubstrateWeight<Runtime>;
  // Wrapped currency
  type CurrencyWrapr = Adapter<AccountId>;
  type PeriodBasis = PeriodBasis;
  // Asset registry
  type AssetRegistry = WraprAssetRegistry;
}

impl pallet_quorum::Config for Runtime {
  type Event = Event;
  type QuorumPalletId = QuorumPalletId;
  // FIXME: Use local weight
  type WeightInfo = pallet_quorum::weights::SubstrateWeight<Runtime>;
  // Wrapped currency
  type CurrencyWrapr = Adapter<AccountId>;
  // Security utils
  type Security = WraprSecurity;
  // Asset registry
  type AssetRegistry = WraprAssetRegistry;
}

impl pallet_oracle::Config for Runtime {
  type Event = Event;
  type OraclePalletId = OraclePalletId;
  // FIXME: Use local weight
  type WeightInfo = pallet_oracle::weights::SubstrateWeight<Runtime>;
  // Wrapped currency
  type CurrencyWrapr = Adapter<AccountId>;
  // Fees management
  type Fees = WraprFees;
  // Security utils
  type Security = WraprSecurity;
}

impl pallet_security::Config for Runtime {
  type Event = Event;
}

impl pallet_asset_registry::Config for Runtime {
  type Event = Event;
  type WeightInfo = pallet_asset_registry::weights::SubstrateWeight<Runtime>;
  type AssetRegistryPalletId = AssetRegistryPalletId;
  // Wrapped currency
  type CurrencyWrapr = Adapter<AccountId>;
}

impl pallet_fees::Config for Runtime {
  type Event = Event;
  type FeesPalletId = FeesPalletId;
  type CurrencyWrapr = Adapter<AccountId>;
  type UnixTime = Timestamp;
  // Security utils
  type Security = WraprSecurity;
  type WeightInfo = pallet_fees::weights::SubstrateWeight<Runtime>;
  type ForceOrigin = EnsureOneOf<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollectiveInstance>,
  >;
}
