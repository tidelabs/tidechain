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
    currency::{deposit, UNITS},
    time::{EPOCH_DURATION_IN_SLOTS, MILLISECS_PER_BLOCK},
  },
  types::{
    AccountId, Balance, BlockExecutionWeight, EpochDuration, GrandpaId, ImOnlineId, Moment,
    RuntimeBlockLength, RuntimeBlockWeights,
  },
  Babe, BagsList, Balances, BlockNumber, Call, CouncilCollectiveInstance,
  ElectionProviderMultiPhase, Event, Historical, ImOnline, Offences, Runtime, Session, SessionKeys,
  Staking, Timestamp, TransactionPayment, Treasury,
};
use codec::Decode;
use frame_election_provider_support::{SortedListProvider, VoteWeight};
use frame_support::{
  pallet_prelude::PhantomData,
  parameter_types,
  traits::{ConstU32, EnsureOneOf, KeyOwnerProofSystem, U128CurrencyToVote},
  weights::{DispatchClass, Weight},
};
use frame_system::EnsureRoot;
use sp_core::u32_trait::{_2, _3, _4};
use sp_runtime::{
  curve::PiecewiseLinear,
  traits::{OpaqueKeys, TrailingZeroInput},
  transaction_validity::TransactionPriority,
  KeyTypeId, Perbill,
};

use sp_std::prelude::*;

pub const MAX_NOMINATIONS: u32 =
  <NposCompactSolution16 as sp_npos_elections::NposSolution>::LIMIT as u32;
/// Maximum number of iterations for balancing that will be executed in the embedded miner of
/// pallet-election-provider-multi-phase.
pub const MINER_MAX_ITERATIONS: u32 = 10;

parameter_types! {
   pub const MaxAuthorities: u32 = 100_000;
   // NOTE: Currently it is not possible to change the epoch duration after the chain has started.
   //       Attempting to do so will brick block production.
   pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
   pub const ReportLongevity: u64 =
       BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
}

impl pallet_babe::Config for Runtime {
  type EpochDuration = EpochDuration;
  type ExpectedBlockTime = ExpectedBlockTime;
  type EpochChangeTrigger = pallet_babe::ExternalTrigger;

  type KeyOwnerProofSystem = Historical;

  type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
    KeyTypeId,
    pallet_babe::AuthorityId,
  )>>::Proof;

  type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
    KeyTypeId,
    pallet_babe::AuthorityId,
  )>>::IdentificationTuple;

  type HandleEquivocation =
    pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;
  type WeightInfo = ();
  type DisabledValidators = Session;
  type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
  pub const MaxKeys: u32 = 10_000;
  pub const MaxPeerInHeartbeats: u32 = 10_000;
  pub const MaxPeerDataEncodingSize: u32 = 1_000;
  pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
    /// We prioritize im-online heartbeats over election solution submission.
   pub NposSolutionPriority: TransactionPriority =
   Perbill::from_percent(90) * TransactionPriority::max_value();
}

impl pallet_im_online::Config for Runtime {
  type AuthorityId = ImOnlineId;
  type Event = Event;
  type NextSessionRotation = Babe;
  type ValidatorSet = Historical;
  type ReportUnresponsiveness = Offences;
  type UnsignedPriority = ImOnlineUnsignedPriority;
  type MaxKeys = MaxKeys;
  type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
  type MaxPeerDataEncodingSize = MaxPeerDataEncodingSize;
  type WeightInfo = crate::weights::pallet_im_online::WeightInfo<Runtime>;
}

impl pallet_authority_discovery::Config for Runtime {
  type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
   pub const UncleGenerations: BlockNumber = 0;
}

impl pallet_authorship::Config for Runtime {
  type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
  type UncleGenerations = UncleGenerations;
  type FilterUncle = ();
  type EventHandler = (Staking, ImOnline);
}

impl pallet_offences::Config for Runtime {
  type Event = Event;
  type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
  type OnOffenceHandler = Staking;
}

impl pallet_grandpa::Config for Runtime {
  type Event = Event;
  type Call = Call;

  type KeyOwnerProofSystem = Historical;

  type KeyOwnerProof =
    <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

  type KeyOwnerIdentification =
    <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::IdentificationTuple;

  type HandleEquivocation =
    pallet_grandpa::EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;

  type MaxAuthorities = MaxAuthorities;
  type WeightInfo = crate::weights::pallet_grandpa::WeightInfo<Runtime>;
}

impl pallet_session::Config for Runtime {
  type Event = Event;
  type ValidatorId = <Self as frame_system::Config>::AccountId;
  type ValidatorIdOf = pallet_staking::StashOf<Self>;
  type ShouldEndSession = Babe;
  type NextSessionRotation = Babe;
  type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
  type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
  type Keys = SessionKeys;
  type WeightInfo = crate::weights::pallet_session::WeightInfo<Runtime>;
}

impl pallet_session::historical::Config for Runtime {
  type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
  type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

parameter_types! {
  // phase durations. 1/4 of the last session for each.
  pub const SignedPhase: u32 = EPOCH_DURATION_IN_SLOTS / 4;
  pub const UnsignedPhase: u32 = EPOCH_DURATION_IN_SLOTS / 4 ;

  // signed config
  pub const SignedMaxSubmissions: u32 = 16;
  // 40 TIDE fixed deposit..
  pub const SignedDepositBase: Balance = deposit(2, 0);
  // 0.01 TIDE per KB of solution data.
  pub const SignedDepositByte: Balance = deposit(0, 10) / 1024;
  // Each good submission will get 1 TIDE as reward
  pub SignedRewardBase: Balance = UNITS;
  pub SolutionImprovementThreshold: Perbill = Perbill::from_rational(5u32, 10_000);

  // miner configs
  pub OffchainRepeat: BlockNumber = 5;
}

sp_npos_elections::generate_solution_type!(
  #[compact]
  pub struct NposCompactSolution16::<
    VoterIndex = u32,
    TargetIndex = u16,
    Accuracy = sp_runtime::PerU16,
  >(16)
);

/// A source of random balance for the NPoS Solver, which is meant to be run by the off-chain worker
/// election miner.
pub struct OffchainRandomBalancing;
impl frame_support::pallet_prelude::Get<Option<(usize, sp_npos_elections::ExtendedBalance)>>
  for OffchainRandomBalancing
{
  fn get() -> Option<(usize, sp_npos_elections::ExtendedBalance)> {
    let iters = match MINER_MAX_ITERATIONS {
      0 => 0,
      max => {
        let seed = sp_io::offchain::random_seed();
        let random = <u32>::decode(&mut TrailingZeroInput::new(&seed))
          .expect("input is padded with zeroes; qed")
          % max.saturating_add(1);
        random as usize
      }
    };

    Some((iters, 0))
  }
}

/// The numbers configured here could always be more than the the maximum limits of staking pallet
/// to ensure election snapshot will not run out of memory. For now, we set them to smaller values
/// since the staking is bounded and the weight pipeline takes hours for this single pallet.
pub struct BenchmarkConfigMultiPhase;
impl pallet_election_provider_multi_phase::BenchmarkingConfig for BenchmarkConfigMultiPhase {
  const VOTERS: [u32; 2] = [100, 200];
  const TARGETS: [u32; 2] = [50, 100];
  const ACTIVE_VOTERS: [u32; 2] = [50, 80];
  const DESIRED_TARGETS: [u32; 2] = [20, 40];
  const SNAPSHOT_MAXIMUM_VOTERS: u32 = 100;
  const MINER_MAXIMUM_VOTERS: u32 = 100;
  const MAXIMUM_TARGETS: u32 = 30;
}

impl pallet_election_provider_multi_phase::Config for Runtime {
  type Event = Event;
  type Currency = Balances;
  type EstimateCallFee = TransactionPayment;
  type UnsignedPhase = UnsignedPhase;
  type SignedMaxSubmissions = SignedMaxSubmissions;
  type SignedRewardBase = SignedRewardBase;
  type SignedDepositBase = SignedDepositBase;
  type SignedDepositByte = SignedDepositByte;
  type SignedDepositWeight = ();
  type SignedMaxWeight = Self::MinerMaxWeight;
  type SlashHandler = (); // burn slashes
  type RewardHandler = (); // nothing to do upon rewards
  type SignedPhase = SignedPhase;
  type SolutionImprovementThreshold = SolutionImprovementThreshold;
  type MinerMaxWeight = OffchainSolutionWeightLimit; // For now use the one from staking.
  type MinerMaxLength = OffchainSolutionLengthLimit;
  type OffchainRepeat = OffchainRepeat;
  type MinerTxPriority = NposSolutionPriority;
  type DataProvider = Staking;
  type Solution = NposCompactSolution16;
  type Fallback = pallet_election_provider_multi_phase::NoFallback<Self>;
  type Solver = frame_election_provider_support::SequentialPhragmen<
    AccountId,
    pallet_election_provider_multi_phase::SolutionAccuracyOf<Self>,
    OffchainRandomBalancing,
  >;
  type BenchmarkingConfig = BenchmarkConfigMultiPhase;
  type ForceOrigin = EnsureOneOf<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollectiveInstance>,
  >;
  type VoterSnapshotPerBlock = VoterSnapshotPerBlock;
  type WeightInfo = crate::weights::pallet_election_provider_multi_phase::WeightInfo<Runtime>;
}

parameter_types! {
  /// A limit for off-chain phragmen unsigned solution submission.
    ///
    /// We want to keep it as high as possible, but can't risk having it reject,
    /// so we always subtract the base block execution weight.
  pub OffchainSolutionWeightLimit: Weight = RuntimeBlockWeights::get()
    .get(DispatchClass::Normal)
    .max_extrinsic
    .expect("Normal extrinsics have weight limit configured by default; qed")
    .saturating_sub(BlockExecutionWeight::get());

  /// A limit for off-chain phragmen unsigned solution length.
    ///
    /// We allow up to 90% of the block's size to be consumed by the solution.
  pub OffchainSolutionLengthLimit: u32 = Perbill::from_rational(90_u32, 100) *
    *RuntimeBlockLength::get()
    .max
    .get(DispatchClass::Normal);

  /// Whilst `UseNominatorsAndUpdateBagsList` or `UseNominatorsMap` is in use, this can still be a
  /// very large value. Once the `BagsList` is in full motion, staking might open its door to many
  /// more nominators, and this value should instead be what is a "safe" number (e.g. 22500).
  pub const VoterSnapshotPerBlock: u32 = 22_500;
}

impl frame_election_provider_support::onchain::Config for Runtime {
  type Accuracy = Perbill;
  type DataProvider = Staking;
}

pallet_staking_reward_curve::build! {
   const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
       min_inflation: 0_025_000,
       max_inflation: 0_100_000,
       // Before, we launch the products we want 50% of supply to be staked
       ideal_stake: 0_500_000,
       falloff: 0_050_000,
       max_piece_count: 40,
       test_precision: 0_005_000,
   );
}

parameter_types! {
   // Six sessions in an era (6 hours).
   pub const SessionsPerEra: sp_staking::SessionIndex = 6;
   // 28 eras for unbonding (7 days).
   pub const BondingDuration: pallet_staking::EraIndex = 28;
   // 27 eras in which slashes can be cancelled (slightly less than 7 days).
   pub const SlashDeferDuration: pallet_staking::EraIndex = 27;
   pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
   pub const MaxNominatorRewardedPerValidator: u32 = 256;
   pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

/// A reasonable benchmarking config for staking pallet.
pub struct StakingBenchmarkingConfig;
impl pallet_staking::BenchmarkingConfig for StakingBenchmarkingConfig {
  type MaxValidators = ConstU32<1000>;
  type MaxNominators = ConstU32<1000>;
}

impl pallet_staking::Config for Runtime {
  const MAX_NOMINATIONS: u32 = MAX_NOMINATIONS;
  type Currency = Balances;
  type UnixTime = Timestamp;
  type CurrencyToVote = U128CurrencyToVote;
  type RewardRemainder = Treasury;
  type Event = Event;
  type Slash = Treasury;
  type Reward = ();
  type SessionsPerEra = SessionsPerEra;
  type BondingDuration = BondingDuration;
  type ElectionProvider = ElectionProviderMultiPhase;
  type GenesisElectionProvider =
    frame_election_provider_support::onchain::OnChainSequentialPhragmen<Self>;
  type SlashDeferDuration = SlashDeferDuration;
  /// A super-majority of the council can cancel the slash.
  type SlashCancelOrigin = EnsureOneOf<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollectiveInstance>,
  >;
  type SessionInterface = Self;
  type EraPayout = pallet_staking::ConvertCurve<RewardCurve>;
  type NextNewSession = Session;
  type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
  type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
  // Use the nominators map to iter voters, but also keep bags-list up-to-date.
  type SortedListProvider = BagsList;
  type BenchmarkingConfig = StakingBenchmarkingConfig;
  type WeightInfo = crate::weights::pallet_staking::WeightInfo<Runtime>;
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

/// Implementation of `frame_election_provider_support::SortedListProvider` that updates the
/// bags-list but uses [`pallet_staking::Nominators`] for `iter`. This is meant to be a transitionary
/// implementation for runtimes to "test" out the bags-list by keeping it up to date, but not yet
/// using it for snapshot generation. In contrast, a  "complete" implementation would use bags-list
/// for `iter`.
pub struct UseNominatorsAndUpdateBagsList<T>(PhantomData<T>);
impl<T: pallet_bags_list::Config + pallet_staking::Config> SortedListProvider<T::AccountId>
  for UseNominatorsAndUpdateBagsList<T>
{
  type Error = pallet_bags_list::Error;

  fn iter() -> Box<dyn Iterator<Item = T::AccountId>> {
    Box::new(pallet_staking::Nominators::<T>::iter().map(|(n, _)| n))
  }

  fn count() -> u32 {
    pallet_bags_list::Pallet::<T>::count()
  }

  fn contains(id: &T::AccountId) -> bool {
    pallet_bags_list::Pallet::<T>::contains(id)
  }

  fn on_insert(id: T::AccountId, weight: VoteWeight) -> Result<(), Self::Error> {
    pallet_bags_list::Pallet::<T>::on_insert(id, weight)
  }

  fn on_update(id: &T::AccountId, new_weight: VoteWeight) {
    pallet_bags_list::Pallet::<T>::on_update(id, new_weight);
  }

  fn on_remove(id: &T::AccountId) {
    pallet_bags_list::Pallet::<T>::on_remove(id);
  }

  fn unsafe_regenerate(
    all: impl IntoIterator<Item = T::AccountId>,
    weight_of: Box<dyn Fn(&T::AccountId) -> VoteWeight>,
  ) -> u32 {
    pallet_bags_list::Pallet::<T>::unsafe_regenerate(all, weight_of)
  }

  fn sanity_check() -> Result<(), &'static str> {
    pallet_bags_list::Pallet::<T>::sanity_check()
  }

  fn unsafe_clear() {
    pallet_bags_list::Pallet::<T>::unsafe_clear()
  }

  #[cfg(feature = "runtime-benchmarks")]
  fn weight_update_worst_case(who: &T::AccountId, is_increase: bool) -> VoteWeight {
    pallet_bags_list::Pallet::<T>::weight_update_worst_case(who, is_increase)
  }
}
