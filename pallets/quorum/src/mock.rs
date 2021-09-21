use frame_support::{parameter_types, traits::GenesisBuild, PalletId};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
  testing::Header,
  traits::{BlakeTwo256, IdentityLookup},
};
use system::EnsureRoot;
use tidefi_primitives::AccountId;

use crate::pallet as pallet_quorum;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
  pub enum Test where
    Block = Block,
    NodeBlock = Block,
    UncheckedExtrinsic = UncheckedExtrinsic,
  {
    System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
    Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>},
    Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
    Assets: pallet_assets::<Instance1>::{Pallet, Call, Storage, Event<T>},
    Quorum: pallet_quorum::{Pallet, Call, Config<T>, Storage, Event<T>},
  }
);

parameter_types! {
  pub const BlockHashCount: u64 = 250;
  pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
  type BaseCallFilter = frame_support::traits::Everything;
  type BlockWeights = ();
  type BlockLength = ();
  type DbWeight = ();
  type Origin = Origin;
  type Call = Call;
  type Index = u64;
  type BlockNumber = u64;
  type Hash = H256;
  type Hashing = BlakeTwo256;
  type AccountId = u64;
  type Lookup = IdentityLookup<Self::AccountId>;
  type Header = Header;
  type Event = Event;
  type BlockHashCount = BlockHashCount;
  type Version = ();
  type PalletInfo = PalletInfo;
  type AccountData = pallet_balances::AccountData<Balance>;
  type OnNewAccount = ();
  type OnKilledAccount = ();
  type SystemWeightInfo = ();
  type SS58Prefix = SS58Prefix;
  type OnSetCode = ();
}
pub const TIDE: Balance = 1_000_000_000_000;
parameter_types! {
  pub const ExistentialDeposit: Balance = TIDE;
  pub const MaxLocks: u32 = 50;
  pub const MaxReserves: u32 = 50;
}

parameter_types! {
  pub const AssetDeposit: u64 = 1;
  pub const ApprovalDeposit: u64 = 1;
  pub const StringLimit: u32 = 50;
  pub const MetadataDepositBase: u64 = 1;
  pub const MetadataDepositPerByte: u64 = 1;
}

type AssetsInstance = pallet_assets::Instance1;
impl pallet_assets::Config<AssetsInstance> for Test {
  type Event = Event;
  type Balance = u128;
  type AssetId = u32;
  type Currency = Balances;
  type ForceOrigin = EnsureRoot<AccountId>;
  type AssetDeposit = AssetDeposit;
  type MetadataDepositBase = MetadataDepositBase;
  type MetadataDepositPerByte = MetadataDepositPerByte;
  type ApprovalDeposit = ApprovalDeposit;
  type StringLimit = StringLimit;
  type Freezer = ();
  type Extra = ();
  type WeightInfo = ();
}

impl pallet_balances::Config for Test {
  type Balance = Balance;
  type DustRemoval = ();
  type Event = Event;
  type ExistentialDeposit = ExistentialDeposit;
  type AccountStore = frame_system::Pallet<Test>;
  type MaxLocks = MaxLocks;
  type MaxReserves = MaxReserves;
  type ReserveIdentifier = [u8; 8];
  type WeightInfo = ();
}

parameter_types! {
  pub const WraprPalletId: PalletId = PalletId(*b"wrpr*pal");
}

impl pallet_quorum::Config for Test {
  type Event = Event;
  type WeightInfo = ();
  type QuorumPalletId = WraprPalletId;
}

impl pallet_sudo::Config for Test {
  type Event = Event;
  type Call = Call;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
  let alice = 1u64;
  let mut t = system::GenesisConfig::default()
    .build_storage::<Test>()
    .unwrap();
  pallet_balances::GenesisConfig::<Test>::default()
    .assimilate_storage(&mut t)
    .unwrap();
  pallet_sudo::GenesisConfig::<Test> { key: alice }
    .assimilate_storage(&mut t)
    .unwrap();
  pallet_quorum::GenesisConfig {
    quorum_enabled: false,
    quorum_account: alice,
  }
  .assimilate_storage(&mut t)
  .unwrap();
  t.into()
}
