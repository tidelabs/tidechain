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
  types::{AccountId, AssetId, Balance, BlockNumber, SessionIndex},
  AssetRegistry, AssetRegistryPalletId, Balances, CouncilCollectiveInstance, Event, Fees,
  FeesPalletId, Oracle, OraclePalletId, Origin, Quorum, QuorumPalletId, Runtime, Security,
  TidefiStaking, TidefiStakingPalletId, Timestamp,
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
  // Maximum of 10 stake / currency / user (to prevent bloat on-chain)
  pub const StakeAccountCap: u32 = 10;
  // Maximum unstake processed in queue
  pub const UnstakeQueueCap: u32 = 100;
  // Staking: Number of sessions per era
  // ~ 1 month
  pub const SessionsPerEra: SessionIndex = 720;
  // Staking: Number of sessions to keep in archive
  pub const SessionsArchive: SessionIndex = 5;
  // Staking: Number of block per sessions
  // ~ 5 mins
  pub const BlocksPerSession: BlockNumber = 50;
  // Staking: Number of blocks to wait before unstaking when we force-unstake.
  pub const BlocksForceUnstake: BlockNumber = 100;

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
         || Some(caller.clone()) == AssetRegistry::account_id()
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

impl pallet_tidefi::Config for Runtime {
  type Event = Event;
  type Quorum = Quorum;
  type Oracle = Oracle;
  // FIXME: Use local weight
  type WeightInfo = pallet_tidefi::weights::SubstrateWeight<Runtime>;
  // Wrapped currency
  type CurrencyTidefi = Adapter<AccountId>;
  // Asset registry
  type AssetRegistry = AssetRegistry;
}

impl pallet_tidefi_stake::Config for Runtime {
  type Event = Event;
  type StakePalletId = TidefiStakingPalletId;
  // FIXME: Use local weight
  type WeightInfo = pallet_tidefi_stake::weights::SubstrateWeight<Runtime>;
  // Wrapped currency
  type CurrencyTidefi = Adapter<AccountId>;
  type StakeAccountCap = StakeAccountCap;
  type UnstakeQueueCap = UnstakeQueueCap;
  type BlocksForceUnstake = BlocksForceUnstake;
  // Asset registry
  type AssetRegistry = AssetRegistry;
  type Security = Security;
}

impl pallet_quorum::Config for Runtime {
  type Event = Event;
  type QuorumPalletId = QuorumPalletId;
  // FIXME: Use local weight
  type WeightInfo = pallet_quorum::weights::SubstrateWeight<Runtime>;
  // Wrapped currency
  type CurrencyTidefi = Adapter<AccountId>;
  // Security utils
  type Security = Security;
  // Asset registry
  type AssetRegistry = AssetRegistry;
}

impl pallet_oracle::Config for Runtime {
  type Event = Event;
  type OraclePalletId = OraclePalletId;
  // FIXME: Use local weight
  type WeightInfo = pallet_oracle::weights::SubstrateWeight<Runtime>;
  // Wrapped currency
  type CurrencyTidefi = Adapter<AccountId>;
  // Fees management
  type Fees = Fees;
  // Security utils
  type Security = Security;
}

impl pallet_security::Config for Runtime {
  type Event = Event;
}

impl pallet_asset_registry::Config for Runtime {
  type Event = Event;
  type WeightInfo = pallet_asset_registry::weights::SubstrateWeight<Runtime>;
  type AssetRegistryPalletId = AssetRegistryPalletId;
  // Wrapped currency
  type CurrencyTidefi = Adapter<AccountId>;
}

impl pallet_fees::Config for Runtime {
  type Event = Event;
  type FeesPalletId = FeesPalletId;
  type CurrencyTidefi = Adapter<AccountId>;
  type UnixTime = Timestamp;
  type SessionsPerEra = SessionsPerEra;
  type SessionsArchive = SessionsArchive;
  type BlocksPerSession = BlocksPerSession;
  type Staking = TidefiStaking;
  // Security utils
  type Security = Security;
  type WeightInfo = pallet_fees::weights::SubstrateWeight<Runtime>;
  type ForceOrigin = EnsureOneOf<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollectiveInstance>,
  >;
}
