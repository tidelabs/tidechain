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
  bag_thresholds::THRESHOLDS,
  constants::{
    currency::{deposit, CENTS, MILLICENTS, TIDE},
    fee::WeightToFee,
    time::SLOT_DURATION,
  },
  impls::DealWithFees,
  types::{
    AccountId, AccountIndex, Balance, BlakeTwo256, BlockHashCount, BlockNumber,
    EnsureRootOrHalfCouncil, Hash, Nonce, RocksDbWeight, RuntimeBlockLength, RuntimeBlockWeights,
  },
  Babe, Balances, Call, Event, Indices, Moment, Origin, OriginCaller, PalletInfo, Preimage,
  Runtime, Staking, System, Treasury, SS58_PREFIX, VERSION,
};

use frame_support::{
  parameter_types,
  traits::{ConstU32, Everything, PrivilegeCmp},
  weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_transaction_payment::{CurrencyAdapter, Multiplier, TargetedFeeAdjustment};
use sp_runtime::{FixedPointNumber, Perbill, Perquintill};
use sp_std::cmp::Ordering;
use sp_version::RuntimeVersion;

parameter_types! {
  pub const Version: RuntimeVersion = VERSION;
  pub const SS58Prefix: u8 = SS58_PREFIX;
}

impl frame_system::Config for Runtime {
  type BaseCallFilter = Everything;
  type BlockWeights = RuntimeBlockWeights;
  type BlockLength = RuntimeBlockLength;
  type DbWeight = RocksDbWeight;
  type Origin = Origin;
  type Call = Call;
  type Index = Nonce;
  type BlockNumber = BlockNumber;
  type Hash = Hash;
  type Hashing = BlakeTwo256;
  type AccountId = AccountId;
  type Lookup = Indices;
  type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
  type Event = Event;
  type BlockHashCount = BlockHashCount;
  type Version = Version;
  type PalletInfo = PalletInfo;
  type AccountData = pallet_balances::AccountData<Balance>;
  type OnNewAccount = ();
  type OnKilledAccount = ();
  type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
  type SS58Prefix = SS58Prefix;
  type OnSetCode = ();
  type MaxConsumers = ConstU32<16>;
}

parameter_types! {
   pub const TransactionByteFee: Balance = 10 * MILLICENTS;
   pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
   pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
   pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
   /// This value increases the priority of `Operational` transactions by adding
   /// a "virtual tip" that's equal to the `OperationalFeeMultiplier * final_fee`.
   pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
  type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees<Runtime>>;
  type TransactionByteFee = TransactionByteFee;
  type OperationalFeeMultiplier = OperationalFeeMultiplier;
  type WeightToFee = WeightToFee;
  type FeeMultiplierUpdate =
    TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
}

parameter_types! {
  pub const ExistentialDeposit: Balance = 100 * CENTS;
  pub const MaxLocks: u32 = 50;
  pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
  type Balance = Balance;
  type DustRemoval = ();
  type Event = Event;
  type ExistentialDeposit = ExistentialDeposit;
  type AccountStore = System;
  type MaxLocks = MaxLocks;
  type MaxReserves = MaxReserves;
  type ReserveIdentifier = [u8; 8];
  type WeightInfo = crate::weights::pallet_balances::WeightInfo<Runtime>;
}

parameter_types! {
   pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
       RuntimeBlockWeights::get().max_block;
   pub const MaxScheduledPerBlock: u32 = 50;
   pub const NoPreimagePostponement: Option<u32> = Some(10);
}

/// Used the compare the privilege of an origin inside the scheduler.
pub struct OriginPrivilegeCmp;

impl PrivilegeCmp<OriginCaller> for OriginPrivilegeCmp {
  fn cmp_privilege(left: &OriginCaller, right: &OriginCaller) -> Option<Ordering> {
    if left == right {
      return Some(Ordering::Equal);
    }

    match (left, right) {
      // Root is greater than anything.
      (OriginCaller::system(frame_system::RawOrigin::Root), _) => Some(Ordering::Greater),
      // Check which one has more yes votes.
      (
        OriginCaller::Council(pallet_collective::RawOrigin::Members(l_yes_votes, l_count)),
        OriginCaller::Council(pallet_collective::RawOrigin::Members(r_yes_votes, r_count)),
      ) => Some((l_yes_votes * r_count).cmp(&(r_yes_votes * l_count))),
      // For every other origin we don't care, as they are not used for `ScheduleOrigin`.
      _ => None,
    }
  }
}

impl pallet_scheduler::Config for Runtime {
  type Event = Event;
  type Origin = Origin;
  type PalletsOrigin = OriginCaller;
  type Call = Call;
  type MaximumWeight = MaximumSchedulerWeight;
  type ScheduleOrigin = EnsureRoot<AccountId>;
  type MaxScheduledPerBlock = MaxScheduledPerBlock;
  type OriginPrivilegeCmp = OriginPrivilegeCmp;
  type PreimageProvider = Preimage;
  type NoPreimagePostponement = NoPreimagePostponement;
  type WeightInfo = crate::weights::pallet_scheduler::WeightInfo<Runtime>;
}

parameter_types! {
   pub const IndexDeposit: Balance = TIDE;
}

impl pallet_indices::Config for Runtime {
  type AccountIndex = AccountIndex;
  type Currency = Balances;
  type Deposit = IndexDeposit;
  type Event = Event;
  type WeightInfo = crate::weights::pallet_indices::WeightInfo<Runtime>;
}

parameter_types! {
   pub const MinimumPeriod: Moment = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
  type Moment = Moment;
  type OnTimestampSet = Babe;
  type MinimumPeriod = MinimumPeriod;
  type WeightInfo = crate::weights::pallet_timestamp::WeightInfo<Runtime>;
}

parameter_types! {
    // 258 bytes on-chain
   pub const BasicDeposit: Balance = deposit(1,258);
    // 66 bytes on-chain
   pub const FieldDeposit: Balance = deposit(0,66);
   // 53 bytes on-chain
   pub const SubAccountDeposit: Balance = deposit(1,53);
   pub const MaxSubAccounts: u32 = 100;
   pub const MaxAdditionalFields: u32 = 100;
   pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
  type Event = Event;
  type Currency = Balances;
  type BasicDeposit = BasicDeposit;
  type FieldDeposit = FieldDeposit;
  type SubAccountDeposit = SubAccountDeposit;
  type MaxSubAccounts = MaxSubAccounts;
  type MaxAdditionalFields = MaxAdditionalFields;
  type MaxRegistrars = MaxRegistrars;
  type Slashed = Treasury;
  type ForceOrigin = EnsureRootOrHalfCouncil;
  type RegistrarOrigin = EnsureRootOrHalfCouncil;
  type WeightInfo = crate::weights::pallet_identity::WeightInfo<Runtime>;
}

parameter_types! {
  pub const BagThresholds: &'static [u64] = &THRESHOLDS;
}

impl pallet_bags_list::Config for Runtime {
  type Event = Event;
  type VoteWeightProvider = Staking;
  type BagThresholds = BagThresholds;
  /// FIXME: Revert local weighting
  type WeightInfo = pallet_bags_list::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
  pub const PreimageMaxSize: u32 = 4096 * 1024;
  pub const PreimageBaseDeposit: Balance = deposit(2, 64);
  pub const PreimageByteDeposit: Balance = deposit(0, 1);
}

impl pallet_preimage::Config for Runtime {
  type Event = Event;
  type Currency = Balances;
  type ManagerOrigin = EnsureRoot<AccountId>;
  type MaxSize = PreimageMaxSize;
  type BaseDeposit = PreimageBaseDeposit;
  type ByteDeposit = PreimageByteDeposit;
  type WeightInfo = crate::weights::pallet_preimage::WeightInfo<Runtime>;
}

parameter_types! {
  pub const ConfigDepositBase: Balance = 5 * TIDE;
  pub const FriendDepositFactor: Balance = 50 * CENTS;
  pub const MaxFriends: u16 = 9;
  pub const RecoveryDeposit: Balance = 5 * TIDE;
}

impl pallet_recovery::Config for Runtime {
  type Event = Event;
  type Call = Call;
  type Currency = Balances;
  type ConfigDepositBase = ConfigDepositBase;
  type FriendDepositFactor = FriendDepositFactor;
  type MaxFriends = MaxFriends;
  type RecoveryDeposit = RecoveryDeposit;
}
