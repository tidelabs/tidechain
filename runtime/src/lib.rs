#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
#![allow(clippy::from_over_into)]
#![allow(clippy::manual_range_contains)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
  construct_runtime, parameter_types,
  traits::{
    fungible::{
      Inspect as FungibleInspect, Mutate as FungibleMutate, Transfer as FungibleTransfer,
    },
    fungibles::{Inspect, Mutate, Transfer},
    tokens::{DepositConsequence, WithdrawConsequence},
    EnsureOrigin, Imbalance, KeyOwnerProofSystem, LockIdentifier, U128CurrencyToVote,
  },
  weights::{
    constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
    DispatchClass, Weight,
  },
  RuntimeDebug,
};

use constants::{currency::*, time::*};
use frame_support::{
  traits::{Everything, InstanceFilter, OnUnbalanced},
  weights::{WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial},
  PalletId,
};
#[cfg(any(feature = "std", test))]
pub use frame_system::Call as SystemCall;
use frame_system::{
  limits::{BlockLength, BlockWeights},
  EnsureOneOf, EnsureRoot, RawOrigin,
};
#[cfg(any(feature = "std", test))]
pub use pallet_balances::Call as BalancesCall;
use pallet_grandpa::{
  fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as pallet_session_historical;
#[cfg(any(feature = "std", test))]
pub use pallet_staking::StakerStatus;
pub use pallet_transaction_payment::{CurrencyAdapter, Multiplier, TargetedFeeAdjustment};
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};
use smallvec::smallvec;
use sp_api::impl_runtime_apis;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_core::{
  crypto::KeyTypeId,
  u32_trait::{_1, _2, _3, _4, _5},
  OpaqueMetadata,
};
use sp_inherents::{CheckInherentsResult, InherentData};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
  create_runtime_str,
  curve::PiecewiseLinear,
  generic, impl_opaque_keys,
  traits::{
    self, AccountIdConversion, BlakeTwo256, Block as BlockT, BlockNumberProvider, NumberFor,
    OpaqueKeys, SaturatedConversion, StaticLookup,
  },
  transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
  ApplyExtrinsicResult, DispatchError, DispatchResult, FixedPointNumber, Perbill, Percent, Permill,
  Perquintill,
};
use sp_std::{marker::PhantomData, prelude::*};
#[cfg(any(feature = "std", test))]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use static_assertions::const_assert;
pub use tidefi_primitives::{
  AccountId, AccountIndex, Amount, AssetId, Balance, BalanceInfo, BlockNumber, CurrencyId, Hash,
  Index, Moment, Signature, Stake,
};

/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod impls;

/// Constant values used within the runtime.
pub mod constants;
mod weights;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Wasm binary unwrapped. If built with `SKIP_WASM_BUILD`, the function panics.
#[cfg(feature = "std")]
pub fn wasm_binary_unwrap() -> &'static [u8] {
  WASM_BINARY.expect(
    "Development wasm binary is not available. This means the client is \
						built with `SKIP_WASM_BUILD` flag and it is only usable for \
						production chains. Please rebuild with the flag disabled.",
  )
}

/// Runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
  spec_name: create_runtime_str!("node"),
  impl_name: create_runtime_str!("tidefi-official"),
  authoring_version: 10,
  // Per convention: if the runtime behavior changes, increment spec_version
  // and set impl_version to 0. If only runtime
  // implementation changes and behavior does not, then leave spec_version as
  // is and increment impl_version.
  spec_version: 268,
  impl_version: 0,
  apis: RUNTIME_API_VERSIONS,
  transaction_version: 2,
};

/// The BABE epoch configuration at genesis.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
  sp_consensus_babe::BabeEpochConfiguration {
    c: PRIMARY_PROBABILITY,
    allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
  };

/// Native version.
#[cfg(any(feature = "std", test))]
pub fn native_version() -> NativeVersion {
  NativeVersion {
    runtime_version: VERSION,
    can_author_with: Default::default(),
  }
}

type NegativeImbalance =
  <Balances as frame_support::traits::Currency<AccountId>>::NegativeImbalance;

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
  fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
    if let Some(mut fees) = fees_then_tips.next() {
      if let Some(tips) = fees_then_tips.next() {
        tips.merge_into(&mut fees);
      }
      // Sent everything to treasury
      Treasury::on_unbalanced(fees);
    }
  }
}

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 4 seconds of compute with a 12 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = 4 * WEIGHT_PER_SECOND;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;
    pub RuntimeBlockLength: BlockLength =
        BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
        .base_block(BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
        .for_class(DispatchClass::Normal, |weights| {
            weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
        })
        .for_class(DispatchClass::Operational, |weights| {
            weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
            // Operational transactions have some extra reserved space, so that they
            // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
            weights.reserved = Some(
                MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
            );
        })
        .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();
    pub const SS58Prefix: u8 = 42;
}

const_assert!(NORMAL_DISPATCH_RATIO.deconstruct() >= AVERAGE_ON_INITIALIZE_RATIO.deconstruct());

impl frame_system::Config for Runtime {
  type BaseCallFilter = Everything;
  type BlockWeights = RuntimeBlockWeights;
  type BlockLength = RuntimeBlockLength;
  type DbWeight = RocksDbWeight;
  type Origin = Origin;
  type Call = Call;
  type Index = Index;
  type BlockNumber = BlockNumber;
  type Hash = Hash;
  type Hashing = BlakeTwo256;
  type AccountId = AccountId;
  type Lookup = Indices;
  type Header = generic::Header<BlockNumber, BlakeTwo256>;
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
}

impl pallet_utility::Config for Runtime {
  type Event = Event;
  type Call = Call;
  type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
}

parameter_types! {
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const DepositBase: Balance = deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);
    pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
  type Event = Event;
  type Call = Call;
  type Currency = Balances;
  type DepositBase = DepositBase;
  type DepositFactor = DepositFactor;
  type MaxSignatories = MaxSignatories;
  type WeightInfo = weights::pallet_multisig::WeightInfo<Runtime>;
}

parameter_types! {
    // One storage item; key size 32, value size 8; .
    pub const ProxyDepositBase: Balance = deposit(1, 8);
    // Additional storage item size of 33 bytes.
    pub const ProxyDepositFactor: Balance = deposit(0, 33);
    pub const MaxProxies: u16 = 32;
    pub const AnnouncementDepositBase: Balance = deposit(1, 8);
    pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
    pub const MaxPending: u16 = 32;
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
  Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, MaxEncodedLen,
)]
pub enum ProxyType {
  Any = 0_isize,
  NonTransfer,
  Governance,
  Staking,
}

impl Default for ProxyType {
  fn default() -> Self {
    Self::Any
  }
}

impl InstanceFilter<Call> for ProxyType {
  fn filter(&self, c: &Call) -> bool {
    match self {
      ProxyType::Any => false,
      ProxyType::NonTransfer => !matches!(
        c,
        Call::Balances(..) | Call::Indices(pallet_indices::Call::transfer(..))
      ),
      ProxyType::Governance => matches!(
        c,
        Call::Council(..) | Call::TechnicalCommittee(..) | Call::Elections(..) | Call::Treasury(..)
      ),
      ProxyType::Staking => matches!(c, Call::Staking(..)),
    }
  }
  fn is_superset(&self, o: &Self) -> bool {
    match (self, o) {
      (x, y) if x == y => true,
      (ProxyType::Any, _) => true,
      (_, ProxyType::Any) => false,
      (ProxyType::NonTransfer, _) => true,
      _ => false,
    }
  }
}

impl pallet_proxy::Config for Runtime {
  type Event = Event;
  type Call = Call;
  type Currency = Balances;
  type ProxyType = ProxyType;
  type ProxyDepositBase = ProxyDepositBase;
  type ProxyDepositFactor = ProxyDepositFactor;
  type MaxProxies = MaxProxies;
  type WeightInfo = weights::pallet_proxy::WeightInfo<Runtime>;
  type MaxPending = MaxPending;
  type CallHasher = BlakeTwo256;
  type AnnouncementDepositBase = AnnouncementDepositBase;
  type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
        RuntimeBlockWeights::get().max_block;
    pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
  type Event = Event;
  type Origin = Origin;
  type PalletsOrigin = OriginCaller;
  type Call = Call;
  type MaximumWeight = MaximumSchedulerWeight;
  type ScheduleOrigin = EnsureRoot<AccountId>;
  type MaxScheduledPerBlock = MaxScheduledPerBlock;
  type WeightInfo = weights::pallet_scheduler::WeightInfo<Runtime>;
}

parameter_types! {
    // NOTE: Currently it is not possible to change the epoch duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS as u64;
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
}

parameter_types! {
    pub const IndexDeposit: Balance = TIDE;
}

impl pallet_indices::Config for Runtime {
  type AccountIndex = AccountIndex;
  type Currency = Balances;
  type Deposit = IndexDeposit;
  type Event = Event;
  type WeightInfo = weights::pallet_indices::WeightInfo<Runtime>;
}

parameter_types! {
  pub const ExistentialDeposit: Balance = TIDE;
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
  type WeightInfo = weights::pallet_balances::WeightInfo<Runtime>;
}

parameter_types! {
    pub const TransactionByteFee: Balance = 10 * MILLICENTS;
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
    pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

pub struct Adapter<AccountId> {
  phantom: PhantomData<AccountId>,
}

impl Inspect<AccountId> for Adapter<AccountId> {
  type AssetId = CurrencyId;
  type Balance = Balance;

  fn total_issuance(asset: Self::AssetId) -> Self::Balance {
    match asset {
      CurrencyId::Tide => Balances::total_issuance(),
      CurrencyId::Wrapped(asset_id) => Assets::total_issuance(asset_id),
    }
  }

  fn balance(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
    match asset {
      CurrencyId::Tide => Balances::balance(who),
      CurrencyId::Wrapped(asset_id) => Assets::balance(asset_id, who),
    }
  }

  fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
    match asset {
      CurrencyId::Tide => Balances::minimum_balance(),
      CurrencyId::Wrapped(asset_id) => Assets::minimum_balance(asset_id),
    }
  }

  fn reducible_balance(asset: Self::AssetId, who: &AccountId, keep_alive: bool) -> Self::Balance {
    match asset {
      CurrencyId::Tide => Balances::reducible_balance(who, keep_alive),
      CurrencyId::Wrapped(asset_id) => Assets::reducible_balance(asset_id, who, keep_alive),
    }
  }

  fn can_deposit(
    asset: Self::AssetId,
    who: &AccountId,
    amount: Self::Balance,
  ) -> DepositConsequence {
    match asset {
      CurrencyId::Tide => Balances::can_deposit(who, amount),
      CurrencyId::Wrapped(asset_id) => Assets::can_deposit(asset_id, who, amount),
    }
  }

  fn can_withdraw(
    asset: Self::AssetId,
    who: &AccountId,
    amount: Self::Balance,
  ) -> WithdrawConsequence<Self::Balance> {
    match asset {
      CurrencyId::Tide => Balances::can_withdraw(who, amount),
      CurrencyId::Wrapped(asset_id) => Assets::can_withdraw(asset_id, who, amount),
    }
  }
}

impl Mutate<AccountId> for Adapter<AccountId> {
  fn mint_into(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
    match asset {
      CurrencyId::Tide => Balances::mint_into(who, amount),
      CurrencyId::Wrapped(asset_id) => Assets::mint_into(asset_id, who, amount),
    }
  }

  fn burn_from(
    asset: Self::AssetId,
    who: &AccountId,
    amount: Balance,
  ) -> Result<Balance, DispatchError> {
    match asset {
      CurrencyId::Tide => Balances::burn_from(who, amount),
      CurrencyId::Wrapped(asset_id) => Assets::burn_from(asset_id, who, amount),
    }
  }
}

impl Transfer<AccountId> for Adapter<AccountId>
where
  Assets: Transfer<AccountId>,
{
  fn transfer(
    asset: Self::AssetId,
    source: &AccountId,
    dest: &AccountId,
    amount: Self::Balance,
    keep_alive: bool,
  ) -> Result<Balance, DispatchError> {
    match asset {
      CurrencyId::Tide => {
        <Balances as FungibleTransfer<AccountId>>::transfer(source, dest, amount, keep_alive)
      }
      CurrencyId::Wrapped(asset_id) => {
        <Assets as Transfer<AccountId>>::transfer(asset_id, source, dest, amount, keep_alive)
      }
    }
  }
}

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
  type Balance = Balance;
  fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
    // Extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
    let p = CENTS;
    let q = 10 * Balance::from(ExtrinsicBaseWeight::get());
    smallvec![WeightToFeeCoefficient {
      degree: 1,
      negative: false,
      coeff_frac: Perbill::from_rational(p % q, q),
      coeff_integer: p / q,
    }]
  }
}

impl pallet_transaction_payment::Config for Runtime {
  type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees>;
  type TransactionByteFee = TransactionByteFee;
  type WeightToFee = WeightToFee;
  type FeeMultiplierUpdate =
    TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
}

parameter_types! {
    pub const MinimumPeriod: Moment = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
  type Moment = Moment;
  type OnTimestampSet = Babe;
  type MinimumPeriod = MinimumPeriod;
  type WeightInfo = weights::pallet_timestamp::WeightInfo<Runtime>;
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

impl_opaque_keys! {
    pub struct SessionKeys {
        pub grandpa: Grandpa,
        pub babe: Babe,
        pub im_online: ImOnline,
        pub authority_discovery: AuthorityDiscovery,
    }
}

parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
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
  type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
  type WeightInfo = weights::pallet_session::WeightInfo<Runtime>;
}

impl pallet_session::historical::Config for Runtime {
  type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
  type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
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
    // Six session in a an era (24 hrs)
    pub const SessionsPerEra: sp_staking::SessionIndex = 6;
    // 28 era for unbonding (28 days)
    pub const BondingDuration: pallet_staking::EraIndex = 28;
    pub const SlashDeferDuration: pallet_staking::EraIndex = 27;
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const MaxNominatorRewardedPerValidator: u32 = 256;
}

impl pallet_staking::Config for Runtime {
  type Currency = Balances;
  type UnixTime = Timestamp;
  type CurrencyToVote = U128CurrencyToVote;
  type ElectionProvider = ElectionProviderMultiPhase;
  type GenesisElectionProvider =
    frame_election_provider_support::onchain::OnChainSequentialPhragmen<
      pallet_election_provider_multi_phase::OnChainConfig<Self>,
    >;
  const MAX_NOMINATIONS: u32 = MAX_NOMINATIONS;
  type RewardRemainder = Treasury;
  type Event = Event;
  type Slash = Treasury;
  type Reward = ();
  type SessionsPerEra = SessionsPerEra;
  type BondingDuration = BondingDuration;
  type SlashDeferDuration = SlashDeferDuration;
  /// A super-majority of the council can cancel the slash.
  type SlashCancelOrigin = EnsureOneOf<
    AccountId,
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>,
  >;
  type SessionInterface = Self;
  type EraPayout = pallet_staking::ConvertCurve<RewardCurve>;
  type NextNewSession = Session;
  type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
  type WeightInfo = weights::pallet_staking::WeightInfo<Runtime>;
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
  // fallback: emergency phase.
  pub const Fallback: pallet_election_provider_multi_phase::FallbackStrategy =
    pallet_election_provider_multi_phase::FallbackStrategy::Nothing;
  pub SolutionImprovementThreshold: Perbill = Perbill::from_rational(5u32, 10_000);

  // miner configs
  pub const MinerMaxIterations: u32 = 10;
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
pub const MAX_NOMINATIONS: u32 =
  <NposCompactSolution16 as sp_npos_elections::NposSolution>::LIMIT as u32;

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
}

impl pallet_election_provider_multi_phase::Config for Runtime {
  type Event = Event;
  type Currency = Balances;
  type EstimateCallFee = TransactionPayment;
  type SignedPhase = SignedPhase;
  type UnsignedPhase = UnsignedPhase;
  type SignedMaxSubmissions = SignedMaxSubmissions;
  type SignedRewardBase = SignedRewardBase;
  type SignedDepositBase = SignedDepositBase;
  type SignedDepositByte = SignedDepositByte;
  type SignedDepositWeight = ();
  type SignedMaxWeight = Self::MinerMaxWeight;
  type SlashHandler = (); // burn slashes
  type RewardHandler = (); // nothing to do upon rewards
  type SolutionImprovementThreshold = SolutionImprovementThreshold;
  type MinerMaxIterations = MinerMaxIterations;
  type MinerMaxWeight = OffchainSolutionWeightLimit;
  type MinerMaxLength = OffchainSolutionLengthLimit;
  type OffchainRepeat = OffchainRepeat;
  type MinerTxPriority = NposSolutionPriority;
  type DataProvider = Staking;
  type OnChainAccuracy = Perbill;
  type Fallback = Fallback;
  type BenchmarkingConfig = ();
  type ForceOrigin = EnsureOneOf<
    AccountId,
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>,
  >;
  type WeightInfo = weights::pallet_election_provider_multi_phase::WeightInfo<Runtime>;
  type Solution = NposCompactSolution16;
}

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 7 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
  type Origin = Origin;
  type Proposal = Call;
  type Event = Event;
  type MotionDuration = CouncilMotionDuration;
  type MaxProposals = CouncilMaxProposals;
  type MaxMembers = CouncilMaxMembers;
  type DefaultVote = pallet_collective::PrimeDefaultVote;
  type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
}

parameter_types! {
    pub const CandidacyBond: Balance = 100 * TIDE;
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
  type WeightInfo = weights::pallet_elections_phragmen::WeightInfo<Runtime>;
}

parameter_types! {
    pub const TechnicalMotionDuration: BlockNumber = 7 * DAYS;
    pub const TechnicalMaxProposals: u32 = 100;
    pub const TechnicalMaxMembers: u32 = 100;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
  type Origin = Origin;
  type Proposal = Call;
  type Event = Event;
  type MotionDuration = TechnicalMotionDuration;
  type MaxProposals = TechnicalMaxProposals;
  type MaxMembers = TechnicalMaxMembers;
  type DefaultVote = pallet_collective::PrimeDefaultVote;
  type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
}

type EnsureRootOrHalfCouncil = EnsureOneOf<
  AccountId,
  EnsureRoot<AccountId>,
  pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
>;

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
  type WeightInfo = weights::pallet_membership::WeightInfo<Runtime>;
}

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: Balance = 100 * TIDE;
    pub const SpendPeriod: BlockNumber = 24 * DAYS;
    pub const Burn: Permill = Permill::from_percent(1);
    pub const TipCountdown: BlockNumber = DAYS;
    pub const TipFindersFee: Percent = Percent::from_percent(20);
    pub const TipReportDepositBase: Balance = TIDE;
    pub const DataDepositPerByte: Balance = CENTS;
    pub const BountyDepositBase: Balance = TIDE;
    pub const BountyDepositPayoutDelay: BlockNumber = 8 * DAYS;
    pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
    pub const BountyUpdatePeriod: BlockNumber = 90 * DAYS;
    pub const MaximumReasonLength: u32 = 16384;
    pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
    pub const BountyValueMinimum: Balance = 10 * TIDE;
    pub const MaxApprovals: u32 = 100;
}

impl pallet_treasury::Config for Runtime {
  type PalletId = TreasuryPalletId;
  type Currency = Balances;
  type ApproveOrigin = EnsureOneOf<
    AccountId,
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilCollective>,
  >;
  type RejectOrigin = EnsureRootOrHalfCouncil;
  type Event = Event;
  type OnSlash = ();
  type ProposalBond = ProposalBond;
  type ProposalBondMinimum = ProposalBondMinimum;
  type SpendPeriod = SpendPeriod;
  type Burn = Burn;
  type BurnDestination = ();
  type SpendFunds = Bounties;
  type WeightInfo = weights::pallet_treasury::WeightInfo<Runtime>;
  type MaxApprovals = MaxApprovals;
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
  type WeightInfo = weights::pallet_bounties::WeightInfo<Runtime>;
}

impl pallet_sudo::Config for Runtime {
  type Event = Event;
  type Call = Call;
}

parameter_types! {
    pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
     /// We prioritize im-online heartbeats over election solution submission.
    pub NposSolutionPriority: TransactionPriority =
    Perbill::from_percent(90) * TransactionPriority::max_value();
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
  Call: From<LocalCall>,
{
  fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
    call: Call,
    public: <Signature as traits::Verify>::Signer,
    account: AccountId,
    nonce: Index,
  ) -> Option<(
    Call,
    <UncheckedExtrinsic as traits::Extrinsic>::SignaturePayload,
  )> {
    let tip = 0;
    // take the biggest period possible.
    let period = BlockHashCount::get()
      .checked_next_power_of_two()
      .map(|c| c / 2)
      .unwrap_or(2) as u64;
    let current_block = System::block_number()
      .saturated_into::<u64>()
      // The `System::block_number` is initialized with `n+1`,
      // so the actual block number is `n`.
      .saturating_sub(1);
    let extra = (
      frame_system::CheckSpecVersion::<Runtime>::new(),
      frame_system::CheckTxVersion::<Runtime>::new(),
      frame_system::CheckGenesis::<Runtime>::new(),
      frame_system::CheckMortality::<Runtime>::from(generic::Era::mortal(period, current_block)),
      frame_system::CheckNonce::<Runtime>::from(nonce),
      frame_system::CheckWeight::<Runtime>::new(),
      pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
    );
    let raw_payload = SignedPayload::new(call, extra)
      .map_err(|e| {
        log::warn!("Unable to create signed payload: {:?}", e);
      })
      .ok()?;
    let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
    let address = Indices::unlookup(account);
    let (call, extra, _) = raw_payload.deconstruct();
    Some((call, (address, signature, extra)))
  }
}

impl frame_system::offchain::SigningTypes for Runtime {
  type Public = <Signature as traits::Verify>::Signer;
  type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
  Call: From<C>,
{
  type Extrinsic = UncheckedExtrinsic;
  type OverarchingCall = Call;
}

impl pallet_im_online::Config for Runtime {
  type AuthorityId = ImOnlineId;
  type Event = Event;
  type NextSessionRotation = Babe;
  type ValidatorSet = Historical;
  type ReportUnresponsiveness = Offences;
  type UnsignedPriority = ImOnlineUnsignedPriority;
  type WeightInfo = weights::pallet_im_online::WeightInfo<Runtime>;
}

impl pallet_offences::Config for Runtime {
  type Event = Event;
  type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
  type OnOffenceHandler = Staking;
}
parameter_types! {
      pub const MaxAuthorities: u32 = 100_000;
}

impl pallet_authority_discovery::Config for Runtime {
  type MaxAuthorities = MaxAuthorities;
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

  type WeightInfo = weights::pallet_grandpa::WeightInfo<Runtime>;
}

parameter_types! {
    pub const BasicDeposit: Balance = deposit(1,258);       // 258 bytes on-chain
    pub const FieldDeposit: Balance = deposit(0,66);        // 66 bytes on-chain
    pub const SubAccountDeposit: Balance = deposit(1,53);   // 53 bytes on-chain
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
  type WeightInfo = weights::pallet_identity::WeightInfo<Runtime>;
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

parameter_types! {
  pub MinVestedTransfer: Balance = TIDE;
  pub const MaxVestingSchedules: u32 = 300;
}

pub struct SusbtrateBlockNumberProvider;
impl BlockNumberProvider for SusbtrateBlockNumberProvider {
  type BlockNumber = BlockNumber;

  fn current_block_number() -> Self::BlockNumber {
    System::block_number()
  }
}

pub struct EnsureRootOrTreasury;
impl EnsureOrigin<Origin> for EnsureRootOrTreasury {
  type Success = AccountId;

  fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
    Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
      RawOrigin::Root => Ok(TreasuryPalletId::get().into_account()),
      RawOrigin::Signed(caller) => {
        if caller == TreasuryPalletId::get().into_account() {
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
    Origin::from(RawOrigin::Signed(Default::default()))
  }
}

parameter_types! {
  pub const ApprovalDeposit: Balance = 10 * CENTS;
  pub const AssetDeposit: Balance = DOLLARS;
  pub const AssetsStringLimit: u32 = 50;
  /// Key = 32 bytes, Value = 36 bytes (32+1+1+1+1)
  // https://github.com/paritytech/substrate/blob/069917b/frame/assets/src/lib.rs#L257L271
  pub const MetadataDepositBase: Balance = deposit(1, 68);
  pub const MetadataDepositPerByte: Balance = deposit(0, 1);
  pub const WraprPalletId: PalletId = PalletId(*b"py/wrapr");
  pub const QuorumPalletId: PalletId = PalletId(*b"py/quorm");
  pub const OraclePalletId: PalletId = PalletId(*b"py/oracl");

  pub const PeriodBasis: BlockNumber = 1000u32;
}

impl pallet_assets::Config for Runtime {
  type Event = Event;
  type Balance = Balance;
  type AssetId = AssetId;
  type Currency = Balances;
  type ForceOrigin = EnsureRootOrHalfCouncil;
  type AssetDeposit = AssetDeposit;
  type MetadataDepositBase = MetadataDepositBase;
  type MetadataDepositPerByte = MetadataDepositPerByte;
  type ApprovalDeposit = ApprovalDeposit;
  type StringLimit = AssetsStringLimit;
  type Freezer = ();
  // FIXME: Use local weight
  type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
  type Extra = ();
}

impl pallet_wrapr::Config for Runtime {
  type Event = Event;
  type PalletId = WraprPalletId;
  type Quorum = WraprQuorum;
  type Oracle = WraprOracle;
  // FIXME: Use local weight
  type WeightInfo = pallet_wrapr::weights::SubstrateWeight<Runtime>;
  // Wrapped currency
  type CurrencyWrapr = Adapter<AccountId>;
}

impl pallet_wrapr_stake::Config for Runtime {
  type Event = Event;
  type PalletId = WraprPalletId;
  type Assets = Assets;
  // FIXME: Use local weight
  type WeightInfo = pallet_wrapr_stake::weights::SubstrateWeight<Runtime>;
  // Wrapped currency
  type CurrencyWrapr = Adapter<AccountId>;
  type PeriodBasis = PeriodBasis;
}

impl pallet_quorum::Config for Runtime {
  type Event = Event;
  type QuorumPalletId = QuorumPalletId;
  // FIXME: Use local weight
  type WeightInfo = pallet_quorum::weights::SubstrateWeight<Runtime>;
  // Wrapped currency
  type CurrencyWrapr = Adapter<AccountId>;
}

impl pallet_oracle::Config for Runtime {
  type Event = Event;
  type OraclePalletId = OraclePalletId;
  // FIXME: Use local weight
  type WeightInfo = pallet_oracle::weights::SubstrateWeight<Runtime>;
  // Wrapped currency
  type CurrencyWrapr = Adapter<AccountId>;
}

impl pallet_security::Config for Runtime {
  type Event = Event;
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = tidefi_primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
        Utility: pallet_utility::{Pallet, Call, Event} = 1,
        Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned} = 2,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 3,
        Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent} = 4,
        Indices: pallet_indices::{Pallet, Call, Storage, Config<T>, Event<T>} = 5,
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 6,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 7,
        ElectionProviderMultiPhase: pallet_election_provider_multi_phase::{Pallet, Call, Storage, Event<T>, ValidateUnsigned} = 8,
        Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>} = 9,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 10,
        Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 11,
        TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 12,
        Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>} = 13,
        TechnicalMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 14,
        Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event, ValidateUnsigned} = 15,
        Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 16,
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 17,
        ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>} = 18,
        AuthorityDiscovery: pallet_authority_discovery::{Pallet, Config} = 19,
        Offences: pallet_offences::{Pallet, Storage, Event} = 20,
        Historical: pallet_session_historical::{Pallet} = 21,
        Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 23,
        Recovery: pallet_recovery::{Pallet, Call, Storage, Event<T>} = 24,
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 25,
        Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 26,
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 27,
        Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>} = 28,
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>} = 29,
        // Request handler for withdrawls
        Wrapr: pallet_wrapr::{Pallet, Call, Storage, Event<T>} = 30,
        // Staking storage and hooks
        WraprStake: pallet_wrapr_stake::{Pallet, Call, Storage, Event<T>} = 31,
        // Storage, events and traits for the quorum
        WraprQuorum: pallet_quorum::{Pallet, Call, Config<T>, Storage, Event<T>} = 32,
        // Storage, events and traits for the oracle
        WraprOracle: pallet_oracle::{Pallet, Call, Config<T>, Storage, Event<T>} = 33,
        // Storage, events and traits for the oracle
        WraprSecurity: pallet_security::{Pallet, Call, Config<T>, Storage, Event<T>} = 34,
    }
);
/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;
/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, AccountIndex>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
///
/// When you change this, you **MUST** modify [`sign`] in `bin/node/testing/src/keyring.rs`!
///
/// [`sign`]: <../../testing/src/keyring.rs.html>
pub type SignedExtra = (
  frame_system::CheckSpecVersion<Runtime>,
  frame_system::CheckTxVersion<Runtime>,
  frame_system::CheckGenesis<Runtime>,
  frame_system::CheckMortality<Runtime>,
  frame_system::CheckNonce<Runtime>,
  frame_system::CheckWeight<Runtime>,
  pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
  Runtime,
  Block,
  frame_system::ChainContext<Runtime>,
  Runtime,
  AllPallets,
  (),
>;

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block);
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx,block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> fg_primitives::SetId {
          Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Grandpa::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((fg_primitives::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(fg_primitives::OpaqueKeyOwnershipProof::new)
        }
    }

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            sp_consensus_babe::BabeGenesisConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: BABE_GENESIS_EPOCH_CONFIG.c,
                genesis_authorities: Babe::authorities(),
                randomness: Babe::randomness(),
                allowed_slots: BABE_GENESIS_EPOCH_CONFIG.allowed_slots,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            Babe::current_epoch_start()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            Babe::current_epoch()
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            Babe::next_epoch()
        }

        fn generate_key_ownership_proof(
            _slot: sp_consensus_babe::Slot,
            authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Babe::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }
    }

    impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
        fn authorities() -> Vec<AuthorityDiscoveryId> {
            AuthorityDiscovery::authorities()
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }


    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
        Block,
        Balance,
    > for Runtime {
        fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    // Wrapr Custom API
    impl pallet_wrapr_rpc_runtime_api::WraprApi<Block, AccountId> for Runtime {
      fn get_account_balance(asset_id: CurrencyId, account_id: AccountId) -> Result<BalanceInfo, DispatchError> {
        Wrapr::get_account_balance(asset_id, &account_id)
      }
      fn get_account_stakes(account_id: AccountId) -> Result<Vec<(CurrencyId, Stake<BalanceInfo>)>, DispatchError> {
        Ok(WraprStake::get_account_stakes(&account_id))
      }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        fn on_runtime_upgrade() -> Result<(Weight, Weight), sp_runtime::RuntimeString> {
            let weight = Executive::try_runtime_upgrade()?;
            Ok((weight, RuntimeBlockWeights::get().max_block))
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
      fn benchmark_metadata(extra: bool) -> (
        Vec<frame_benchmarking::BenchmarkList>,
        Vec<frame_support::traits::StorageInfo>,
      ) {
        use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
        use frame_support::traits::StorageInfoTrait;

        use pallet_session_benchmarking::Pallet as SessionBench;
        use pallet_offences_benchmarking::Pallet as OffencesBench;
        use frame_system_benchmarking::Pallet as SystemBench;

        let mut list = Vec::<BenchmarkList>::new();

        list_benchmark!(list, extra, pallet_balances, Balances);
        list_benchmark!(list, extra, pallet_bounties, Bounties);
        list_benchmark!(list, extra, pallet_collective, Council);
        list_benchmark!(list, extra, pallet_election_provider_multi_phase, ElectionProviderMultiPhase);
        list_benchmark!(list, extra, pallet_elections_phragmen, Elections);
        //list_benchmark!(list, extra, pallet_grandpa, Grandpa);
        list_benchmark!(list, extra, pallet_identity, Identity);
        list_benchmark!(list, extra, pallet_im_online, ImOnline);
        list_benchmark!(list, extra, pallet_indices, Indices);
        list_benchmark!(list, extra, pallet_membership, TechnicalMembership);
        list_benchmark!(list, extra, pallet_multisig, Multisig);
        list_benchmark!(list, extra, pallet_offences, OffencesBench::<Runtime>);
        list_benchmark!(list, extra, pallet_proxy, Proxy);
        list_benchmark!(list, extra, pallet_scheduler, Scheduler);
        list_benchmark!(list, extra, pallet_session, SessionBench::<Runtime>);
        list_benchmark!(list, extra, pallet_staking, Staking);
        list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
        list_benchmark!(list, extra, pallet_assets, Assets);
        list_benchmark!(list, extra, pallet_timestamp, Timestamp);
        list_benchmark!(list, extra, pallet_treasury, Treasury);
        list_benchmark!(list, extra, pallet_utility, Utility);
        list_benchmark!(list, extra, pallet_wrapr, Wrapr);
        list_benchmark!(list, extra, pallet_wrapr_stake, WraprStake);
        list_benchmark!(list, extra, pallet_quorum, WraprQuorum);
        list_benchmark!(list, extra, pallet_oracle, WraprOracle);
        list_benchmark!(list, extra, pallet_oracle, WraprSecurity);

        let storage_info = AllPalletsWithSystem::storage_info();

        return (list, storage_info)
      }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};
            // Trying to add benchmarks directly to the Session Pallet caused cyclic dependency
            // issues. To get around that, we separated the Session benchmarks into its own crate,
            // which is why we need these two lines below.
            use pallet_session_benchmarking::Pallet as SessionBench;
            use pallet_offences_benchmarking::Pallet as OffencesBench;
            use frame_system_benchmarking::Pallet as SystemBench;

            impl pallet_session_benchmarking::Config for Runtime {}
            impl pallet_offences_benchmarking::Config for Runtime {}
            impl frame_system_benchmarking::Config for Runtime {}

            let whitelist: Vec<TrackedStorageKey> = vec![
                // Block Number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
                // Total Issuance
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
                // Execution Phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
                // Event Count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
                // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
                // Treasury Account
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmark!(params, batches, pallet_assets, Assets);
            add_benchmark!(params, batches, pallet_balances, Balances);
            add_benchmark!(params, batches, pallet_bounties, Bounties);
            add_benchmark!(params, batches, pallet_collective, Council);
            add_benchmark!(params, batches, pallet_election_provider_multi_phase, ElectionProviderMultiPhase);
            add_benchmark!(params, batches, pallet_elections_phragmen, Elections);
            // FIXME: grandme benchmark do not generate the correct functions
            //add_benchmark!(params, batches, pallet_grandpa, Grandpa);
            add_benchmark!(params, batches, pallet_identity, Identity);
            add_benchmark!(params, batches, pallet_im_online, ImOnline);
            add_benchmark!(params, batches, pallet_indices, Indices);
            add_benchmark!(params, batches, pallet_membership, TechnicalMembership);
            add_benchmark!(params, batches, pallet_multisig, Multisig);
            add_benchmark!(params, batches, pallet_offences, OffencesBench::<Runtime>);
            add_benchmark!(params, batches, pallet_proxy, Proxy);
            add_benchmark!(params, batches, pallet_scheduler, Scheduler);
            add_benchmark!(params, batches, pallet_session, SessionBench::<Runtime>);
            add_benchmark!(params, batches, pallet_staking, Staking);
            add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
            add_benchmark!(params, batches, pallet_timestamp, Timestamp);
            add_benchmark!(params, batches, pallet_treasury, Treasury);
            add_benchmark!(params, batches, pallet_utility, Utility);
            add_benchmark!(params, batches, pallet_wrapr, Wrapr);
            add_benchmark!(params, batches, pallet_wrapr_stake, WraprStake);
            add_benchmark!(params, batches, pallet_quorum, WraprQuorum);
            add_benchmark!(params, batches, pallet_oracle, WraprOracle);
            add_benchmark!(params, batches, pallet_oracle, WraprSecurity);

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}

#[cfg(test)]
mod tests {
  use super::*;
  use frame_system::offchain::CreateSignedTransaction;

  #[test]
  fn validate_transaction_submitter_bounds() {
    fn is_submit_signed_transaction<T>()
    where
      T: CreateSignedTransaction<Call>,
    {
    }

    is_submit_signed_transaction::<Runtime>();
  }
}
