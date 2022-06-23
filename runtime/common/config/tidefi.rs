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
  constants::currency::{deposit, Adapter, TDFY},
  types::{AccountId, AssetId, Balance, BlockNumber, SessionIndex},
  AssetRegistry, AssetRegistryPalletId, Balances, CouncilCollectiveInstance, Event, Fees,
  FeesPalletId, Oracle, OraclePalletId, Origin, Quorum, QuorumPalletId, Runtime, Security,
  TidefiStaking, TidefiStakingPalletId, Timestamp,
};

use frame_support::{
  parameter_types,
  traits::{ConstU128, EitherOfDiverse, EnsureOrigin},
};
use frame_system::{EnsureRoot, RawOrigin};
use sp_runtime::{traits::AccountIdConversion, Permill};

parameter_types! {
  pub const ApprovalDeposit: Balance = TDFY;
  pub const AssetDeposit: Balance = TDFY;
  pub const AssetsStringLimit: u32 = 50;
  /// Key = 32 bytes, Value = 36 bytes (32+1+1+1+1)
  // https://github.com/paritytech/substrate/blob/069917b/frame/assets/src/lib.rs#L257L271
  pub const MetadataDepositBase: Balance = deposit(1, 68);
  pub const MetadataDepositPerByte: Balance = deposit(0, 1);
  // Maximum of 10 stake / currency / user (to prevent bloat on-chain)
  pub const StakeAccountCap: u32 = 10;
  // Maximum unstake processed in queue
  pub const UnstakeQueueCap: u32 = 100;
  // Staking: Number of sessions per era
  // ~ 1 hour
  pub const SessionsPerEra: SessionIndex = 12;
  // Staking: Number of sessions to keep in archive
  pub const SessionsArchive: SessionIndex = 5;
  // Staking: Number of block per sessions
  // ~ 5 mins
  pub const BlocksPerSession: BlockNumber = 50;
  // Staking: Number of blocks to wait before unstaking when we force-unstake.
  pub const BlocksForceUnstake: BlockNumber = 14_400;
  // 0.25%
  pub const FeeAmount: Permill = Permill::from_parts(2500);
  // 0.10%
  pub const MarketMakerFeeAmount: Permill = Permill::from_parts(1000);
  // 0.05%
  pub const MarketMakerLimitFeeAmount: Permill = Permill::from_parts(500);
  // Maximum pending burned possible in queue
  pub const BurnedCap: u32 = 1000;
  // Maximum proposals in queue for the quorum, to limit the vector size and optimization
  pub const ProposalsCap: u32 = 1000;
  // The lifetime of a proposal by the quorum members
  pub const ProposalLifetime: BlockNumber = 100;
  // Maximum number of staking period the chain can support
  pub const StakingRewardCap: u32 = 10;
  // The maximum size of a string
  pub const StringLimit: u32 = 255;
  // The number of votes maximum per proposal, should alway be higher than the proposals threshold
  pub const VotesLimit: u32 = 10;
  // The maximum number of account the watchlist can contains
  pub const WatchListLimit: u32 = 10_000;
  // The maximum number of pubkey each asset can have, should alway be more more than the current quorum active member set
  pub const PubkeyLimitPerAsset: u32 = 10;
  // The number of swap each account can have in queue
  pub const SwapLimitByAccount: u32 = 10_000;
  // Sunrise Pool: Number of blocks to wait before they can claim the last era reward.
  // current_era.start_block + BlocksSunriseClaims < current_block to be able to claim last era sunrise reward
  pub const BlocksSunriseClaims: BlockNumber = 10;
}

pub struct EnsureRootOrAssetRegistry;
impl EnsureOrigin<Origin> for EnsureRootOrAssetRegistry {
  type Success = AccountId;

  fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
    Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
      RawOrigin::Root => Ok(AssetRegistryPalletId::get().into_account_truncating()),
      RawOrigin::Signed(caller) => {
        // Allow call from asset registry pallet ID account
        if caller == AssetRegistryPalletId::get().into_account_truncating()
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
      AssetRegistryPalletId::get().into_account_truncating(),
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
  type Extra = ();
  type WeightInfo = crate::weights::pallet_assets::WeightInfo<Runtime>;
}

impl pallet_tidefi::Config for Runtime {
  type Event = Event;
  type Quorum = Quorum;
  type Oracle = Oracle;
  // Wrapped currency
  type CurrencyTidefi = Adapter<AccountId>;
  // Asset registry
  type AssetRegistry = AssetRegistry;
  type WeightInfo = crate::weights::pallet_tidefi::WeightInfo<Runtime>;
}

impl pallet_tidefi_stake::Config for Runtime {
  type Event = Event;
  type StakePalletId = TidefiStakingPalletId;
  // Wrapped currency
  type CurrencyTidefi = Adapter<AccountId>;
  type StakeAccountCap = StakeAccountCap;
  type UnstakeQueueCap = UnstakeQueueCap;
  type BlocksForceUnstake = BlocksForceUnstake;
  // Asset registry
  type AssetRegistry = AssetRegistry;
  type Security = Security;
  type StakingRewardCap = StakingRewardCap;
  type WeightInfo = crate::weights::pallet_tidefi_stake::WeightInfo<Runtime>;
}

impl pallet_quorum::Config for Runtime {
  type Event = Event;
  type QuorumPalletId = QuorumPalletId;
  // Wrapped currency
  type CurrencyTidefi = Adapter<AccountId>;
  // Security utils
  type Security = Security;
  // Asset registry
  type AssetRegistry = AssetRegistry;
  type ProposalsCap = ProposalsCap;
  type BurnedCap = BurnedCap;
  type ProposalLifetime = ProposalLifetime;
  type StringLimit = StringLimit;
  type VotesLimit = VotesLimit;
  type WatchListLimit = WatchListLimit;
  type PubkeyLimitPerAsset = PubkeyLimitPerAsset;
  type WeightInfo = crate::weights::pallet_quorum::WeightInfo<Runtime>;
}

impl pallet_oracle::Config for Runtime {
  type Event = Event;
  type OraclePalletId = OraclePalletId;
  // Wrapped currency
  type CurrencyTidefi = Adapter<AccountId>;
  // Fees management
  type Fees = Fees;
  // Security utils
  type Security = Security;
  type SwapLimitByAccount = SwapLimitByAccount;
  type WeightInfo = crate::weights::pallet_oracle::WeightInfo<Runtime>;
}

impl pallet_security::Config for Runtime {
  type Event = Event;
}

impl pallet_asset_registry::Config for Runtime {
  type Event = Event;
  type AssetRegistryPalletId = AssetRegistryPalletId;
  // Wrapped currency
  type CurrencyTidefi = Adapter<AccountId>;
  type WeightInfo = crate::weights::pallet_asset_registry::WeightInfo<Runtime>;
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
  // Swap fees
  type FeeAmount = FeeAmount;
  type MarketMakerFeeAmount = MarketMakerFeeAmount;
  type MarketMakerLimitFeeAmount = MarketMakerLimitFeeAmount;
  // Security utils
  type Security = Security;
  type BlocksSunriseClaims = BlocksSunriseClaims;
  type WeightInfo = crate::weights::pallet_fees::WeightInfo<Runtime>;
  type ForceOrigin = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollectiveInstance, 2, 3>,
  >;
}
