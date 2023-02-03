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

//! Mocks for the vesting module.

#![cfg(test)]

use super::*;
use frame_support::{
  construct_runtime, parameter_types,
  traits::{ConstU32, ConstU64, EnsureOrigin, Everything},
};
use frame_system::RawOrigin;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};

use crate as vesting;

pub type AccountId = u128;
impl frame_system::Config for Runtime {
  type RuntimeOrigin = RuntimeOrigin;
  type RuntimeCall = RuntimeCall;
  type Index = u64;
  type BlockNumber = u64;
  type Hash = H256;
  type Hashing = ::sp_runtime::traits::BlakeTwo256;
  type AccountId = AccountId;
  type Lookup = IdentityLookup<Self::AccountId>;
  type Header = Header;
  type RuntimeEvent = RuntimeEvent;
  type BlockHashCount = ConstU64<250>;
  type BlockWeights = ();
  type BlockLength = ();
  type Version = ();
  type PalletInfo = PalletInfo;
  type AccountData = pallet_balances::AccountData<u64>;
  type OnNewAccount = ();
  type OnKilledAccount = ();
  type DbWeight = ();
  type BaseCallFilter = Everything;
  type SystemWeightInfo = ();
  type SS58Prefix = ();
  type OnSetCode = ();
  type MaxConsumers = ConstU32<16>;
}

type Balance = u64;

impl pallet_balances::Config for Runtime {
  type Balance = Balance;
  type DustRemoval = ();
  type RuntimeEvent = RuntimeEvent;
  type ExistentialDeposit = ConstU64<1>;
  type AccountStore = frame_system::Pallet<Runtime>;
  type MaxLocks = ();
  type MaxReserves = ();
  type ReserveIdentifier = [u8; 8];
  type WeightInfo = ();
}

pub struct EnsureAliceOrBob;
impl EnsureOrigin<RuntimeOrigin> for EnsureAliceOrBob {
  type Success = AccountId;

  fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
    Into::<Result<RawOrigin<AccountId>, RuntimeOrigin>>::into(o).and_then(|o| match o {
      RawOrigin::Signed(ALICE) => Ok(ALICE),
      RawOrigin::Signed(BOB) => Ok(BOB),
      r => Err(RuntimeOrigin::from(r)),
    })
  }

  #[cfg(feature = "runtime-benchmarks")]
  fn successful_origin() -> RuntimeOrigin {
    let zero_account_id = AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
      .expect("infinite length input; no invalid inputs for type; qed");
    RuntimeOrigin::from(RawOrigin::Signed(zero_account_id))
  }
}

parameter_types! {
  pub static MockBlockNumberProvider: u64 = 0;
}

impl BlockNumberProvider for MockBlockNumberProvider {
  type BlockNumber = u64;

  fn current_block_number() -> Self::BlockNumber {
    Self::get()
  }
}

impl Config for Runtime {
  type RuntimeEvent = RuntimeEvent;
  type Currency = PalletBalances;
  type MinVestedTransfer = ConstU64<5>;
  type VestedTransferOrigin = EnsureAliceOrBob;
  type WeightInfo = crate::weights::SubstrateWeight<Runtime>;
  type MaxVestingSchedules = ConstU32<2>;
  type BlockNumberProvider = MockBlockNumberProvider;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
  pub enum Runtime where
    Block = Block,
    NodeBlock = Block,
    UncheckedExtrinsic = UncheckedExtrinsic,
  {
    System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
    Vesting: vesting::{Pallet, Storage, Call, Event<T>, Config<T>},
    PalletBalances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
  }
);

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;

#[derive(Default)]
pub struct ExtBuilder;

impl ExtBuilder {
  pub fn build() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
      .build_storage::<Runtime>()
      .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
      balances: vec![(ALICE, 100), (CHARLIE, 50)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    vesting::GenesisConfig::<Runtime> {
      vesting: vec![
        // who, start, period, period_count, per_period
        (CHARLIE, 2, 3, 1, 5),
        (CHARLIE, 2 + 3, 3, 3, 5),
      ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
  }
}
