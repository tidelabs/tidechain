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

use frame_support::{
  dispatch::RawOrigin, pallet_prelude::EnsureOrigin, traits::AsEnsureOriginWithArg, PalletId,
};
use frame_system as system;
use frame_utils::construct_mock_runtime;
use sp_runtime::traits::AccountIdConversion;
use system::EnsureRoot;
use tidefi_primitives::{BlockNumber, CurrencyId, SessionIndex};

use crate::pallet as pallet_tidefi;

pub struct EnsureRootOrAssetRegistry;
impl EnsureOrigin<RuntimeOrigin> for EnsureRootOrAssetRegistry {
  type Success = AccountId;

  fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
    Into::<Result<RawOrigin<AccountId>, RuntimeOrigin>>::into(o).and_then(|o| match o {
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
          Err(RuntimeOrigin::from(Some(caller)))
        }
      }
      r => Err(RuntimeOrigin::from(r)),
    })
  }

  #[cfg(feature = "runtime-benchmarks")]
  fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
    Ok(RuntimeOrigin::from(RawOrigin::Signed(
      AssetRegistryPalletId::get().into_account_truncating(),
    )))
  }
}

construct_mock_runtime!({
  Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>},
  Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
  Tidefi: pallet_tidefi::{Pallet, Call, Storage, Event<T>},
  TidefiStaking: pallet_tidefi_stake::{Pallet, Call, Storage, Event<T>},
  Fees: pallet_fees::{Pallet, Storage, Event<T>},
  Sunrise: pallet_sunrise::{Pallet, Config<T>, Storage, Event<T>},
  Quorum: pallet_quorum::{Pallet, Call, Config<T>, Storage, Event<T>},
  Oracle: pallet_oracle::{Pallet, Call, Config<T>, Storage, Event<T>},
  Security: pallet_security::{Pallet, Call, Config, Storage, Event<T>},
  AssetRegistry: pallet_asset_registry::{Pallet, Call, Config<T>, Storage, Event<T>},
}, {
  pub const AssetDeposit: u64 = 1;
  pub const ApprovalDeposit: u64 = 1;
  pub const MetadataDepositBase: u64 = 1;
  pub const MetadataDepositPerByte: u64 = 1;
  pub const QuorumPalletId: PalletId = PalletId(*b"qurm*pal");
  pub const OraclePalletId: PalletId = PalletId(*b"orcl*pal");
  pub const AssetRegistryPalletId: PalletId = PalletId(*b"asst*pal");
  pub const FeesPalletId: PalletId = PalletId(*b"fees*pal");
  pub const SunrisePalletId: PalletId = PalletId(*b"sunr*pal");
  pub const StakePalletId: PalletId = PalletId(*b"stak*pal");
  pub const SessionsPerEra: SessionIndex = 10;
  pub const SessionsArchive: SessionIndex = 2;
  pub const BlocksPerSession: BlockNumber = 50;
  pub const BlocksForceUnstake: BlockNumber = 10;
  pub const StakeAccountCap: u32 = 10;
  // 20 basis point
  pub const FeeAmount: Permill = Permill::from_perthousand(20);
  // 10 basis point
  pub const MarketMakerFeeAmount: Permill = Permill::from_perthousand(10);
  pub const MarketMakerLimitFeeAmount: Permill = Permill::from_parts(500);
  // 20 %
  pub const DistributionPercentage: Permill = Permill::from_percent(20);
  pub const BurnedCap: u32 = 1000;
  // Maximum proposals in queue for the quorum, to limit the vector size and optimization
  pub const ProposalsCap: u32 = 1000;
  // The lifetime of a proposal by the quorum members
  pub const ProposalLifetime: BlockNumber = 100;
  // The maximum size of a string
  pub const StringLimit: u32 = 255;
  // The number of votes maximum per proposal, should alway be higher than the proposals threshold
  pub const VotesLimit: u32 = 10;
  // The maximum number of account the watchlist can contains
  pub const WatchListLimit: u32 = 10000;
  // The maximum number of pubkey each asset can have, should alway be more more than the current quorum active member set
  pub const PubkeyLimitPerAsset: u32 = 10;
  // The number of swap each account can have in queue
  pub const SwapLimitByAccount: u32 = 100;
  // The maximum number of supported market pairs
  pub const SupportedMarketPairsLimit: u8 = 255;
  // Maximum number of staking period the chain can support
  pub const StakingRewardCap: u32 = 10;
  pub const Cooldown: BlockNumber = 1_296_000; // 90 DAYS
  // max 10k rewards
  pub const MaximumRewardPerSwap: Balance = 10_000_000_000_000_000;
  // 50%
  pub const LeftoverSwapRebates: FixedU128 = FixedU128::from_inner(500_000_000_000_000_000);
  pub const BatchSize: u32 = 10;
});

impl pallet_assets::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type Balance = Balance;
  type AssetId = u32;
  type AssetIdParameter = u32;
  type Currency = Balances;
  type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
  type ForceOrigin = EnsureRootOrAssetRegistry;
  type AssetDeposit = AssetDeposit;
  type AssetAccountDeposit = AssetDeposit;
  type MetadataDepositBase = MetadataDepositBase;
  type MetadataDepositPerByte = MetadataDepositPerByte;
  type ApprovalDeposit = ApprovalDeposit;
  type StringLimit = StringLimit;
  type Freezer = ();
  type WeightInfo = ();
  type CallbackHandle = ();
  type Extra = ();
  type RemoveItemsLimit = ConstU32<5>;
  #[cfg(feature = "runtime-benchmarks")]
  type BenchmarkHelper = ();
}

impl pallet_tidefi::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type WeightInfo = crate::weights::SubstrateWeight<Test>;
  type Quorum = Quorum;
  type CurrencyTidefi = Adapter<AccountId>;
  type Oracle = Oracle;
  type Fees = Fees;
  type Sunrise = Sunrise;
  type Security = Security;
  type AssetRegistry = AssetRegistry;
}

impl pallet_quorum::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type WeightInfo = pallet_quorum::weights::SubstrateWeight<Test>;
  type QuorumPalletId = QuorumPalletId;
  type CurrencyTidefi = Adapter<AccountId>;
  type Security = Security;
  type Sunrise = Sunrise;
  type AssetRegistry = AssetRegistry;
  type ProposalsCap = ProposalsCap;
  type BurnedCap = BurnedCap;
  type ProposalLifetime = ProposalLifetime;
  type StringLimit = StringLimit;
  type VotesLimit = VotesLimit;
  type WatchListLimit = WatchListLimit;
  type PubkeyLimitPerAsset = PubkeyLimitPerAsset;
}

impl pallet_oracle::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type WeightInfo = pallet_oracle::weights::SubstrateWeight<Test>;
  type OraclePalletId = OraclePalletId;
  type CurrencyTidefi = Adapter<AccountId>;
  type Security = Security;
  type SwapLimitByAccount = SwapLimitByAccount;
  type SupportedMarketPairsLimit = SupportedMarketPairsLimit;
  type Fees = Fees;
  type Sunrise = Sunrise;
}

impl pallet_security::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type WeightInfo = pallet_security::weights::SubstrateWeight<Test>;
}

impl pallet_asset_registry::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type WeightInfo = pallet_asset_registry::weights::SubstrateWeight<Test>;
  type AssetRegistryPalletId = AssetRegistryPalletId;
  type CurrencyTidefi = Adapter<AccountId>;
}

impl pallet_sudo::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type RuntimeCall = RuntimeCall;
}

impl pallet_fees::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type Security = Security;
  type FeesPalletId = FeesPalletId;
  type CurrencyTidefi = Adapter<AccountId>;
  type ForceOrigin = EnsureRoot<Self::AccountId>;
  type UnixTime = Timestamp;
  type SessionsPerEra = SessionsPerEra;
  type SessionsArchive = SessionsArchive;
  type FeeAmount = FeeAmount;
  type MarketMakerFeeAmount = MarketMakerFeeAmount;
  type MarketMakerLimitFeeAmount = MarketMakerLimitFeeAmount;
  type BlocksPerSession = BlocksPerSession;
  type Staking = TidefiStaking;
  type Sunrise = Sunrise;
  type WeightInfo = pallet_fees::weights::SubstrateWeight<Test>;
}

impl pallet_sunrise::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type Security = Security;
  type SunrisePalletId = SunrisePalletId;
  type CurrencyTidefi = Adapter<AccountId>;
  type Cooldown = Cooldown;
  type MaximumRewardPerSwap = MaximumRewardPerSwap;
  type LeftoverSwapRebates = LeftoverSwapRebates;
}

impl pallet_tidefi_stake::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type WeightInfo = pallet_tidefi_stake::weights::SubstrateWeight<Test>;
  type StakePalletId = StakePalletId;
  type CurrencyTidefi = Adapter<AccountId>;
  type StakeAccountCap = StakeAccountCap;
  type BlocksForceUnstake = BlocksForceUnstake;
  type AssetRegistry = AssetRegistry;
  type StakingRewardCap = StakingRewardCap;
  type Security = Security;
  type BatchSize = BatchSize;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
  let mut storage = system::GenesisConfig::default()
    .build_storage::<Test>()
    .unwrap();

  pallet_balances::GenesisConfig::<Test>::default()
    .assimilate_storage(&mut storage)
    .unwrap();

  pallet_fees::GenesisConfig::<Test>::default()
    .assimilate_storage(&mut storage)
    .unwrap();

  pallet_sunrise::GenesisConfig::<Test>::default()
    .assimilate_storage(&mut storage)
    .unwrap();

  pallet_sudo::GenesisConfig::<Test> {
    key: Some(0_u64.into()),
  }
  .assimilate_storage(&mut storage)
  .unwrap();

  pallet_quorum::GenesisConfig::<Test> {
    enabled: true,
    members: vec![0_u64.into()],
    threshold: 1,
  }
  .assimilate_storage(&mut storage)
  .unwrap();

  pallet_oracle::GenesisConfig::<Test> {
    enabled: true,
    account: 1_u64.into(),
    market_makers: Vec::new(),
    market_pairs: Vec::new(),
  }
  .assimilate_storage(&mut storage)
  .unwrap();

  pallet_asset_registry::GenesisConfig::<Test> {
    assets: vec![(
      CurrencyId::Wrapped(2),
      "Test".into(),
      "TEST".into(),
      8,
      vec![],
    )],
    account: 0_u64.into(),
  }
  .assimilate_storage(&mut storage)
  .unwrap();

  let mut ext: sp_io::TestExternalities = storage.into();
  ext.execute_with(|| System::set_block_number(1));
  ext
}
