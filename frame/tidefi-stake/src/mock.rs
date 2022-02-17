#![allow(dead_code)]
use frame_benchmarking::frame_support::traits::tokens::{DepositConsequence, WithdrawConsequence};
use frame_support::{
  parameter_types,
  traits::{
    fungible::{
      Inspect as FungibleInspect, InspectHold as FungibleInspectHold, Mutate as FungibleMutate,
      MutateHold as FungibleMutateHold, Transfer as FungibleTransfer,
    },
    fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
    ConstU128, ConstU32, GenesisBuild,
  },
  PalletId,
};
use frame_system as system;
#[cfg(feature = "std")]
use sp_core::H256;
use sp_runtime::{
  generic::Header,
  traits::{BlakeTwo256, IdentityLookup},
  DispatchError, DispatchResult,
};
use std::marker::PhantomData;
use system::EnsureRoot;
use tidefi_primitives::{BlockNumber, CurrencyId};

use crate::pallet as pallet_tidefi_stake;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;
type AccountId = u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
  pub enum Test where
    Block = Block,
    NodeBlock = Block,
    UncheckedExtrinsic = UncheckedExtrinsic,
  {
    System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
    Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
    Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
    TidefiStaking: pallet_tidefi_stake::{Pallet, Call, Config<T>, Storage, Event<T>},
    AssetRegistry: pallet_asset_registry::{Pallet, Call, Config<T>, Storage, Event<T>},
    Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
    Security: pallet_security::{Pallet, Call, Config, Storage, Event<T>},
  }
);

parameter_types! {
  pub const BlockHashCount: u32 = 250;
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
  type BlockNumber = BlockNumber;
  type Hash = H256;
  type Hashing = BlakeTwo256;
  type AccountId = AccountId;
  type Lookup = IdentityLookup<Self::AccountId>;
  type Header = Header<BlockNumber, BlakeTwo256>;
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
  type MaxConsumers = ConstU32<16>;
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
  pub const TidefiPalletId: PalletId = PalletId(*b"wrpr*pal");
  pub const QuorumPalletId: PalletId = PalletId(*b"qurm*pal");
  pub const AssetRegistryPalletId: PalletId = PalletId(*b"asst*pal");
  pub const BlocksForceUnstake: BlockNumber = 10;
  pub const MinimumPeriod: u64 = 5;
  pub const StakeAccountCap: u32 = 10;
  pub const UnstakeQueueCap: u32 = 100;
}

impl pallet_assets::Config for Test {
  type Event = Event;
  type Balance = u128;
  type AssetId = u32;
  type Currency = Balances;
  type AssetDeposit = AssetDeposit;
  type MetadataDepositBase = MetadataDepositBase;
  type MetadataDepositPerByte = MetadataDepositPerByte;
  type ApprovalDeposit = ApprovalDeposit;
  type StringLimit = StringLimit;
  type Freezer = ();
  type Extra = ();
  type WeightInfo = ();
  type ForceOrigin = EnsureRoot<Self::AccountId>;
  type AssetAccountDeposit = ConstU128<0>;
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

impl pallet_timestamp::Config for Test {
  type Moment = u64;
  type OnTimestampSet = ();
  type MinimumPeriod = MinimumPeriod;
  type WeightInfo = ();
}

impl pallet_security::Config for Test {
  type Event = Event;
}

impl pallet_tidefi_stake::Config for Test {
  type Event = Event;
  type WeightInfo = crate::weights::SubstrateWeight<Test>;
  type StakePalletId = TidefiPalletId;
  type CurrencyTidefi = Adapter<AccountId>;
  type StakeAccountCap = StakeAccountCap;
  type UnstakeQueueCap = UnstakeQueueCap;
  type BlocksForceUnstake = BlocksForceUnstake;
  type AssetRegistry = AssetRegistry;
  type Security = Security;
}

impl pallet_asset_registry::Config for Test {
  type Event = Event;
  type WeightInfo = pallet_asset_registry::weights::SubstrateWeight<Test>;
  type AssetRegistryPalletId = AssetRegistryPalletId;
  type CurrencyTidefi = Adapter<AccountId>;
}

// this is only the mock for benchmarking, it's implemented directly in the runtime
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

impl InspectHold<AccountId> for Adapter<AccountId> {
  fn balance_on_hold(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
    match asset {
      CurrencyId::Tide => Balances::balance_on_hold(who),
      CurrencyId::Wrapped(asset_id) => Assets::balance_on_hold(asset_id, who),
    }
  }
  fn can_hold(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> bool {
    match asset {
      CurrencyId::Tide => Balances::can_hold(who, amount),
      CurrencyId::Wrapped(asset_id) => Assets::can_hold(asset_id, who, amount),
    }
  }
}

impl MutateHold<AccountId> for Adapter<AccountId> {
  fn hold(asset: CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
    match asset {
      CurrencyId::Tide => Balances::hold(who, amount),
      CurrencyId::Wrapped(asset_id) => Assets::hold(asset_id, who, amount),
    }
  }

  fn release(
    asset: CurrencyId,
    who: &AccountId,
    amount: Balance,
    best_effort: bool,
  ) -> Result<Balance, DispatchError> {
    match asset {
      CurrencyId::Tide => Balances::release(who, amount, best_effort),
      CurrencyId::Wrapped(asset_id) => Assets::release(asset_id, who, amount, best_effort),
    }
  }
  fn transfer_held(
    asset: CurrencyId,
    source: &AccountId,
    dest: &AccountId,
    amount: Balance,
    best_effort: bool,
    on_hold: bool,
  ) -> Result<Balance, DispatchError> {
    match asset {
      CurrencyId::Tide => Balances::transfer_held(source, dest, amount, best_effort, on_hold),
      CurrencyId::Wrapped(asset_id) => {
        Assets::transfer_held(asset_id, source, dest, amount, best_effort, on_hold)
      }
    }
  }
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
  let mut t = system::GenesisConfig::default()
    .build_storage::<Test>()
    .unwrap();

  pallet_balances::GenesisConfig::<Test>::default()
    .assimilate_storage(&mut t)
    .unwrap();

  pallet_tidefi_stake::GenesisConfig::<Test>::default()
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
    account: 0,
  }
  .assimilate_storage(&mut t)
  .unwrap();
  t.into()
}
