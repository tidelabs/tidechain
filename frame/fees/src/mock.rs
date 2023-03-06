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

use crate::pallet as pallet_fees;
use frame_support::{
  dispatch::RawOrigin, pallet_prelude::EnsureOrigin, traits::AsEnsureOriginWithArg, PalletId,
};
use frame_system as system;
use frame_utils::construct_mock_runtime;
use sp_runtime::traits::AccountIdConversion;
use system::EnsureRoot;
use tidefi_primitives::{
  assets, BlockNumber, CurrencyId, SessionIndex, StakeCurrencyMeta, SunriseSwapPool,
};

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
  Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
  AssetRegistry: pallet_asset_registry::{Pallet, Call, Config<T>, Storage, Event<T>},
  Fees: pallet_fees::{Pallet, Config<T>, Storage, Event<T>},
  Sunrise: pallet_sunrise::{Pallet, Config<T>, Storage, Event<T>},
  Security: pallet_security::{Pallet, Call, Config, Storage, Event<T>},
  TidefiStaking: pallet_tidefi_stake::{Pallet, Call, Config<T>, Storage, Event<T>},
}, {
  pub const AssetDeposit: u64 = 1;
  pub const ApprovalDeposit: u64 = 1;
  pub const StringLimit: u32 = 50;
  pub const MetadataDepositBase: u64 = 1;
  pub const MetadataDepositPerByte: u64 = 1;
  pub const SunrisePalletId: PalletId = PalletId(*b"sunr*pal");
  pub const TidefiPalletId: PalletId = PalletId(*b"wrpr*pal");
  pub const QuorumPalletId: PalletId = PalletId(*b"qurm*pal");
  pub const AssetRegistryPalletId: PalletId = PalletId(*b"asst*pal");
  pub const StakePalletId: PalletId = PalletId(*b"stak*pal");
  pub const PeriodBasis: BlockNumber = 1000u32;
  pub const SessionsPerEra: SessionIndex = 10;
  pub const SessionsArchive: SessionIndex = 2;
  pub const BlocksPerSession: BlockNumber = 50;
  pub const BlocksForceUnstake: BlockNumber = 10;
  pub const StakeAccountCap: u32 = 10;
  pub const FeeAmount: Permill = Permill::from_perthousand(20);
  pub const MarketMakerFeeAmount: Permill = Permill::from_perthousand(10);
  pub const MarketMakerLimitFeeAmount: Permill = Permill::from_perthousand(10);
  pub const StakingRewardCap: u32 = 10;
  pub const Cooldown: BlockNumber = 1_296_000; // 90 DAYS
  pub const MaximumRewardPerSwap: Balance = 10_000_000_000_000_000;
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

impl pallet_sunrise::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type Security = Security;
  type SunrisePalletId = SunrisePalletId;
  type CurrencyTidefi = Adapter<AccountId>;
  type Cooldown = Cooldown;
  type MaximumRewardPerSwap = MaximumRewardPerSwap;
  type LeftoverSwapRebates = LeftoverSwapRebates;
}

impl pallet_fees::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type Security = Security;
  type FeesPalletId = TidefiPalletId;
  type CurrencyTidefi = Adapter<AccountId>;
  type ForceOrigin = EnsureRoot<Self::AccountId>;
  type UnixTime = Timestamp;
  type SessionsPerEra = SessionsPerEra;
  type SessionsArchive = SessionsArchive;
  type BlocksPerSession = BlocksPerSession;
  type FeeAmount = FeeAmount;
  type MarketMakerFeeAmount = MarketMakerFeeAmount;
  type MarketMakerLimitFeeAmount = MarketMakerLimitFeeAmount;
  type Staking = TidefiStaking;
  type Sunrise = Sunrise;
  type WeightInfo = crate::weights::SubstrateWeight<Test>;
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

impl pallet_asset_registry::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type WeightInfo = pallet_asset_registry::weights::SubstrateWeight<Test>;
  type AssetRegistryPalletId = AssetRegistryPalletId;
  type CurrencyTidefi = Adapter<AccountId>;
}

impl pallet_security::Config for Test {
  type RuntimeEvent = RuntimeEvent;
  type WeightInfo = pallet_security::weights::SubstrateWeight<Test>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
  let _alice = 1u64;
  let mut t = system::GenesisConfig::default()
    .build_storage::<Test>()
    .unwrap();
  pallet_fees::GenesisConfig::<Test>::default()
    .assimilate_storage(&mut t)
    .unwrap();
  pallet_balances::GenesisConfig::<Test>::default()
    .assimilate_storage(&mut t)
    .unwrap();
  pallet_sunrise::GenesisConfig::<Test> {
    phantom: Default::default(),
    onboarding_rebates: Default::default(),
    swap_pools: vec![
      SunriseSwapPool {
        id: 1,
        minimum_tdfy_value: 0,
        transactions_remaining: 1,
        balance: assets::Asset::Tdfy.saturating_mul(67_200_000),
        // 125%
        rebates: FixedU128::saturating_from_rational(125_u32, 100_u32),
      },
      SunriseSwapPool {
        id: 2,
        minimum_tdfy_value: 1_000_000_000_000_000,
        transactions_remaining: 1,
        balance: assets::Asset::Tdfy.saturating_mul(67_200_000),
        // 200%
        rebates: FixedU128::saturating_from_rational(200_u32, 100_u32),
      },
    ],
  }
  .assimilate_storage(&mut t)
  .unwrap();
  pallet_asset_registry::GenesisConfig::<Test> {
    assets: vec![(
      CurrencyId::Wrapped(2),
      "Test".into(),
      "TEST".into(),
      8,
      vec![],
    )],
    account: AccountId(0),
  }
  .assimilate_storage(&mut t)
  .unwrap();
  pallet_tidefi_stake::GenesisConfig::<Test> {
    unstake_fee: Percent::from_parts(1),
    staking_periods: vec![
      (150_u32.into(), Percent::from_parts(1)),
      ((14400_u32 * 15_u32).into(), Percent::from_parts(2)),
      ((14400_u32 * 30_u32).into(), Percent::from_parts(3)),
      ((14400_u32 * 60_u32).into(), Percent::from_parts(4)),
      ((14400_u32 * 90_u32).into(), Percent::from_parts(5)),
    ],
    staking_meta: vec![(
      CurrencyId::Wrapped(2),
      StakeCurrencyMeta {
        minimum_amount: 100,
        maximum_amount: 500_000_000,
      },
    )],
    operator: AccountId(0),
  }
  .assimilate_storage(&mut t)
  .unwrap();

  t.into()
}
