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
  constants::{
    currency::{deposit, CENTS, DOLLARS, TDFY},
    time::{DAYS, HOURS},
  },
  types::{AccountId, Balance, BlockNumber, EnsureRootOrHalfCouncil},
  Balances, Bounties, Call, Council, CouncilCollectiveInstance, Event, Origin, OriginCaller,
  Runtime, Scheduler, TechnicalCollectiveInstance, TechnicalCommittee, Treasury, TreasuryPalletId,
};
use frame_support::{
  parameter_types,
  traits::{EitherOfDiverse, LockIdentifier, U128CurrencyToVote},
};
use frame_system::EnsureRoot;
use sp_runtime::Permill;
use static_assertions::const_assert;

parameter_types! {
   pub const CandidacyBond: Balance = 100 * TDFY;
   // 1 storage item created, key size is 32 bytes, value size is 16+16.
   pub const VotingBondBase: Balance = deposit(1, 64);
   // additional data per vote is 32 bytes (account id).
   pub const VotingBondFactor: Balance = deposit(0, 32);
   pub const TermDuration: BlockNumber = 7 * DAYS;
   pub const DesiredMembers: u32 = 13;
   pub const DesiredRunnersUp: u32 = 20;
   pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
}

// Make sure that there are no more than `MaxMembers` members elected via elections-phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl pallet_elections_phragmen::Config for Runtime {
  type Event = Event;
  type PalletId = ElectionsPhragmenPalletId;
  type Currency = Balances;
  type ChangeMembers = Council;
  // NOTE: this implies that council's genesis members cannot be set directly and must come from
  // this module.
  type InitializeMembers = Council;
  type CurrencyToVote = U128CurrencyToVote;
  type CandidacyBond = CandidacyBond;
  type VotingBondBase = VotingBondBase;
  type VotingBondFactor = VotingBondFactor;
  type LoserCandidate = ();
  type KickedMember = ();
  type DesiredMembers = DesiredMembers;
  type DesiredRunnersUp = DesiredRunnersUp;
  type TermDuration = TermDuration;
  type WeightInfo = crate::weights::pallet_elections_phragmen::WeightInfo<Runtime>;
}

parameter_types! {
   pub const CouncilMotionDuration: BlockNumber = 7 * DAYS;
   pub const CouncilMaxProposals: u32 = 100;
   pub const CouncilMaxMembers: u32 = 100;
}

impl pallet_collective::Config<CouncilCollectiveInstance> for Runtime {
  type Origin = Origin;
  type Proposal = Call;
  type Event = Event;
  type MotionDuration = CouncilMotionDuration;
  type MaxProposals = CouncilMaxProposals;
  type MaxMembers = CouncilMaxMembers;
  type DefaultVote = pallet_collective::PrimeDefaultVote;
  type WeightInfo = crate::weights::pallet_collective::WeightInfo<Runtime>;
}

parameter_types! {
   pub const TechnicalMotionDuration: BlockNumber = 3 * DAYS;
   pub const TechnicalMaxProposals: u32 = 100;
   pub const TechnicalMaxMembers: u32 = 100;
}

impl pallet_collective::Config<TechnicalCollectiveInstance> for Runtime {
  type Origin = Origin;
  type Proposal = Call;
  type Event = Event;
  type MotionDuration = TechnicalMotionDuration;
  type MaxProposals = TechnicalMaxProposals;
  type MaxMembers = TechnicalMaxMembers;
  type DefaultVote = pallet_collective::PrimeDefaultVote;
  type WeightInfo = crate::weights::pallet_collective::WeightInfo<Runtime>;
}

impl pallet_membership::Config<pallet_membership::Instance1> for Runtime {
  type Event = Event;
  type AddOrigin = EnsureRootOrHalfCouncil;
  type RemoveOrigin = EnsureRootOrHalfCouncil;
  type SwapOrigin = EnsureRootOrHalfCouncil;
  type ResetOrigin = EnsureRootOrHalfCouncil;
  type PrimeOrigin = EnsureRootOrHalfCouncil;
  type MembershipInitialized = TechnicalCommittee;
  type MembershipChanged = TechnicalCommittee;
  type MaxMembers = TechnicalMaxMembers;
  type WeightInfo = crate::weights::pallet_membership::WeightInfo<Runtime>;
}

parameter_types! {
   pub const ProposalBond: Permill = Permill::from_percent(5);
   pub const ProposalBondMinimum: Balance = 100 * DOLLARS;
   pub const ProposalBondMaximum: Balance = 500 * DOLLARS;
   pub const SpendPeriod: BlockNumber = 24 * DAYS;
   pub const Burn: Permill = Permill::from_percent(1);
   pub const DataDepositPerByte: Balance = CENTS;
   pub const BountyDepositBase: Balance = DOLLARS;
   pub const BountyDepositPayoutDelay: BlockNumber = 4 * DAYS;
   pub const BountyUpdatePeriod: BlockNumber = 90 * DAYS;
   pub const MaximumReasonLength: u32 = 16384;
   pub const BountyValueMinimum: Balance = 10 * DOLLARS;
   pub const MaxApprovals: u32 = 100;
   pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
   pub const CuratorDepositMin: Balance = 10 * DOLLARS;
   pub const CuratorDepositMax: Balance = 200 * DOLLARS;
}

impl pallet_treasury::Config for Runtime {
  type PalletId = TreasuryPalletId;
  type Currency = Balances;
  type ApproveOrigin = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollectiveInstance, 4, 5>,
  >;
  type RejectOrigin = EnsureRootOrHalfCouncil;
  type Event = Event;
  type OnSlash = ();
  type ProposalBond = ProposalBond;
  type ProposalBondMinimum = ProposalBondMinimum;
  type ProposalBondMaximum = ProposalBondMaximum;
  type SpendPeriod = SpendPeriod;
  type Burn = Burn;
  type BurnDestination = ();
  type SpendFunds = Bounties;
  type MaxApprovals = MaxApprovals;
  type WeightInfo = crate::weights::pallet_treasury::WeightInfo<Runtime>;
  type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>;
}

impl pallet_bounties::Config for Runtime {
  type BountyDepositBase = BountyDepositBase;
  type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
  type BountyUpdatePeriod = BountyUpdatePeriod;

  type CuratorDepositMultiplier = CuratorDepositMultiplier;
  type CuratorDepositMin = CuratorDepositMin;
  type CuratorDepositMax = CuratorDepositMax;

  type BountyValueMinimum = BountyValueMinimum;
  type ChildBountyManager = ();
  type DataDepositPerByte = DataDepositPerByte;
  type Event = Event;
  type MaximumReasonLength = MaximumReasonLength;
  type WeightInfo = crate::weights::pallet_bounties::WeightInfo<Runtime>;
}

parameter_types! {
  pub LaunchPeriod: BlockNumber = 7 * DAYS;
  pub VotingPeriod: BlockNumber = 7 * DAYS;
  pub FastTrackVotingPeriod: BlockNumber = 3 * HOURS;
  pub const MinimumDeposit: Balance = 100 * CENTS;
  pub EnactmentPeriod: BlockNumber = 8 * DAYS;
  pub CooloffPeriod: BlockNumber = 7 * DAYS;
  pub const InstantAllowed: bool = true;
  pub const MaxVotes: u32 = 100;
  pub const MaxProposals: u32 = 100;
  pub const PreimageByteDeposit: Balance = deposit(0, 1);
}

impl pallet_democracy::Config for Runtime {
  type Proposal = Call;
  type Event = Event;
  type Currency = Balances;
  type EnactmentPeriod = EnactmentPeriod;
  type VoteLockingPeriod = EnactmentPeriod;
  type LaunchPeriod = LaunchPeriod;
  type VotingPeriod = VotingPeriod;
  type MinimumDeposit = MinimumDeposit;
  /// A straight majority of the council can decide what their next motion is.
  type ExternalOrigin =
    pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollectiveInstance, 1, 2>;
  /// A majority can have the next scheduled referendum be a straight majority-carries vote.
  type ExternalMajorityOrigin =
    pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollectiveInstance, 1, 2>;
  /// A unanimous council can have the next scheduled referendum be a straight default-carries
  /// (NTB) vote.
  type ExternalDefaultOrigin =
    pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollectiveInstance, 1, 1>;
  /// Two thirds of the technical committee can have an `ExternalMajority/ExternalDefault` vote
  /// be tabled immediately and with a shorter voting/enactment period.
  type FastTrackOrigin =
    pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollectiveInstance, 2, 3>;
  type InstantOrigin =
    pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollectiveInstance, 1, 1>;
  type InstantAllowed = InstantAllowed;
  type FastTrackVotingPeriod = FastTrackVotingPeriod;
  // To cancel a proposal which has been passed, 2/3 of the council must agree to it.
  type CancellationOrigin = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollectiveInstance, 2, 3>,
  >;
  type BlacklistOrigin = EnsureRoot<AccountId>;
  // To cancel a proposal before it has been passed, the technical committee must be unanimous or
  // Root must agree.
  type CancelProposalOrigin = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollectiveInstance, 1, 1>,
  >;
  // Any single technical committee member may veto a coming council proposal, however they can
  // only do it once and it lasts only for the cooloff period.
  type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollectiveInstance>;
  type CooloffPeriod = CooloffPeriod;
  type PreimageByteDeposit = PreimageByteDeposit;
  type OperationalPreimageOrigin =
    pallet_collective::EnsureMember<AccountId, CouncilCollectiveInstance>;
  type Slash = Treasury;
  type Scheduler = Scheduler;
  type PalletsOrigin = OriginCaller;
  type MaxVotes = MaxVotes;
  type WeightInfo = crate::weights::pallet_democracy::WeightInfo<Runtime>;
  type MaxProposals = MaxProposals;
}
