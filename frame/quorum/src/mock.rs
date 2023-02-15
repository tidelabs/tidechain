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

use crate::pallet as pallet_quorum;
use frame_support::{traits::ConstU128, PalletId};
use frame_system as system;
use frame_utils::construct_mock_runtime;
use system::EnsureRoot;
use tidefi_primitives::{BlockNumber, CurrencyId};

construct_mock_runtime!({
  Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
  Quorum: pallet_quorum::{Pallet, Call, Config<T>, Storage, Event<T>},
  Security: pallet_security::{Pallet, Call, Config, Storage, Event<T>},
  Sunrise: pallet_sunrise::{Pallet, Config<T>, Storage, Event<T>},
  AssetRegistry: pallet_asset_registry::{Pallet, Call, Config<T>, Storage, Event<T>},
}, {
  pub const SunrisePalletId: PalletId = PalletId(*b"sunr*pal");
  pub const TidefiPalletId: PalletId = PalletId(*b"wrpr*pal");
  pub const AssetRegistryPalletId: PalletId = PalletId(*b"asst*pal");
  pub const AssetDeposit: u64 = 1;
  pub const ApprovalDeposit: u64 = 1;
  pub const MetadataDepositBase: u64 = 1;
  pub const MetadataDepositPerByte: u64 = 1;

  // Maximum proposals in queue for the quorum, to limit the vector size and optimization
  pub const ProposalsCap: u32 = 1000;
  pub const BurnedCap: u32 = 1000;

  // The lifetime of a proposal by the quorum members
  pub const ProposalLifetime: u64 = 100;
  // The maximum size of a string
  pub const StringLimit: u32 = 255;
  // The number of votes maximum per proposal, should alway be higher than the proposals threshold
  pub const VotesLimit: u32 = 10;
  // The maximum number of account the watchlist can contains
  pub const WatchListLimit: u32 = 10000;
  // The maximum number of pubkey each asset can have, should alway be more more than the current quorum active member set
  pub const PubkeyLimitPerAsset: u32 = 10;
  pub const Cooldown: BlockNumber = 1_296_000; // 90 DAYS
  // max 10k rewards
  pub const MaximumRewardPerSwap: Balance = 10_000_000_000_000_000;
  // 50%
  pub const LeftoverSwapRebates: FixedU128 = FixedU128::from_inner(500_000_000_000_000_000);

});

impl pallet_assets::Config for Test {
  type Event = Event;
  type Balance = u128;
  type AssetId = u32;
  type Currency = Balances;
  type ForceOrigin = EnsureRoot<Self::AccountId>;
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

impl pallet_quorum::Config for Test {
  type Event = Event;
  type WeightInfo = crate::weights::SubstrateWeight<Test>;
  type QuorumPalletId = TidefiPalletId;
  type Security = Security;
  type Sunrise = Sunrise;
  type CurrencyTidefi = Adapter<AccountId>;
  type AssetRegistry = AssetRegistry;
  type ProposalsCap = ProposalsCap;
  type BurnedCap = BurnedCap;
  type ProposalLifetime = ProposalLifetime;
  type StringLimit = StringLimit;
  type VotesLimit = VotesLimit;
  type WatchListLimit = WatchListLimit;
  type PubkeyLimitPerAsset = PubkeyLimitPerAsset;
}

impl pallet_sunrise::Config for Test {
  type Event = Event;
  type Security = Security;
  type SunrisePalletId = SunrisePalletId;
  type CurrencyTidefi = Adapter<AccountId>;
  type Cooldown = Cooldown;
  type MaximumRewardPerSwap = MaximumRewardPerSwap;
  type LeftoverSwapRebates = LeftoverSwapRebates;
}

impl pallet_security::Config for Test {
  type Event = Event;
  type WeightInfo = pallet_security::weights::SubstrateWeight<Test>;
}

impl pallet_asset_registry::Config for Test {
  type Event = Event;
  type WeightInfo = pallet_asset_registry::weights::SubstrateWeight<Test>;
  type AssetRegistryPalletId = AssetRegistryPalletId;
  type CurrencyTidefi = Adapter<AccountId>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
  let mut storage = system::GenesisConfig::default()
    .build_storage::<Test>()
    .unwrap();
  pallet_balances::GenesisConfig::<Test>::default()
    .assimilate_storage(&mut storage)
    .unwrap();
  pallet_quorum::GenesisConfig::<Test> {
    enabled: false,
    members: vec![1_u64.into()],
    threshold: 1,
  }
  .assimilate_storage(&mut storage)
  .unwrap();

  let mut ext: sp_io::TestExternalities = storage.into();
  ext.execute_with(|| System::set_block_number(1));
  ext
}
