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
    currency::{deposit, CENTS, TIFI},
    time::DAYS,
  },
  types::{AccountId, Balance, BlockNumber, EnsureRootOrHalfCouncil},
  Balances, Bounties, Call, Council, CouncilCollectiveInstance, Event, Origin, Runtime,
  TechnicalCollectiveInstance, TechnicalCommittee, TreasuryPalletId,
};
use frame_support::{
  parameter_types,
  traits::{EnsureOneOf, LockIdentifier, U128CurrencyToVote},
};
use frame_system::EnsureRoot;
use sp_runtime::{Percent, Permill};
use static_assertions::const_assert;

parameter_types! {
   pub const CandidacyBond: Balance = 100 * TIFI;
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
   pub const ProposalBondMinimum: Balance = 100 * TIFI;
   pub const ProposalBondMaximum: Balance = 100_000 * TIFI;
   pub const SpendPeriod: BlockNumber = 24 * DAYS;
   pub const Burn: Permill = Permill::from_percent(1);
   pub const TipCountdown: BlockNumber = DAYS;
   pub const TipFindersFee: Percent = Percent::from_percent(20);
   pub const TipReportDepositBase: Balance = TIFI;
   pub const DataDepositPerByte: Balance = CENTS;
   pub const BountyDepositBase: Balance = TIFI;
   pub const BountyDepositPayoutDelay: BlockNumber = 4 * DAYS;
   pub const BountyUpdatePeriod: BlockNumber = 90 * DAYS;
   pub const MaximumReasonLength: u32 = 16384;
   pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
   pub const BountyValueMinimum: Balance = 10 * TIFI;
   pub const MaxApprovals: u32 = 100;
}

impl pallet_treasury::Config for Runtime {
  type PalletId = TreasuryPalletId;
  type Currency = Balances;
  type ApproveOrigin = EnsureOneOf<
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
}

impl pallet_bounties::Config for Runtime {
  type Event = Event;
  type BountyDepositBase = BountyDepositBase;
  type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
  type BountyUpdatePeriod = BountyUpdatePeriod;
  type BountyCuratorDeposit = BountyCuratorDeposit;
  type BountyValueMinimum = BountyValueMinimum;
  type DataDepositPerByte = DataDepositPerByte;
  type MaximumReasonLength = MaximumReasonLength;
  type ChildBountyManager = ();
  type WeightInfo = crate::weights::pallet_bounties::WeightInfo<Runtime>;
}
