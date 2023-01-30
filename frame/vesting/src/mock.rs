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
  traits::{
    fungible::{
      Inspect as FungibleInspect, Mutate as FungibleMutate, Transfer as FungibleTransfer,
    },
    fungibles::{Inspect, Mutate, Transfer},
    tokens::{DepositConsequence, WithdrawConsequence},
    ConstU128, ConstU32, ConstU64, EnsureOrigin, Everything, GenesisBuild,
  },
  PalletId,
};
use frame_system::{EnsureRoot, RawOrigin};
#[cfg(feature = "std")]
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, DispatchError, DispatchResult};
use std::marker::PhantomData;
use tidefi_primitives::CurrencyId;

use crate as vesting;

pub type AccountId = u128;
impl frame_system::Config for Runtime {
  type Origin = Origin;
  type Call = Call;
  type Index = u64;
  type BlockNumber = u64;
  type Hash = H256;
  type Hashing = ::sp_runtime::traits::BlakeTwo256;
  type AccountId = AccountId;
  type Lookup = IdentityLookup<Self::AccountId>;
  type Header = Header;
  type Event = Event;
  type BlockHashCount = ConstU64<250>;
  type BlockWeights = ();
  type BlockLength = ();
  type Version = ();
  type PalletInfo = PalletInfo;
  type AccountData = pallet_balances::AccountData<u128>;
  type OnNewAccount = ();
  type OnKilledAccount = ();
  type DbWeight = ();
  type BaseCallFilter = Everything;
  type SystemWeightInfo = ();
  type SS58Prefix = ();
  type OnSetCode = ();
  type MaxConsumers = ConstU32<16>;
}

parameter_types! {
  pub const ExistentialDeposit: Balance = 0;
  pub const MaxLocks: u32 = 50;
  pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
  type Balance = Balance;
  type DustRemoval = ();
  type Event = Event;
  type ExistentialDeposit = ExistentialDeposit;
  type AccountStore = frame_system::Pallet<Runtime>;
  type MaxLocks = MaxLocks;
  type MaxReserves = MaxReserves;
  type ReserveIdentifier = [u8; 8];
  type WeightInfo = ();
}

pub struct EnsureAliceOrBob;
impl EnsureOrigin<Origin> for EnsureAliceOrBob {
  type Success = AccountId;

  fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
    Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
      RawOrigin::Signed(ALICE) => Ok(ALICE),
      RawOrigin::Signed(BOB) => Ok(BOB),
      r => Err(Origin::from(r)),
    })
  }

  #[cfg(feature = "runtime-benchmarks")]
  fn successful_origin() -> Origin {
    let zero_account_id = AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
      .expect("infinite length input; no invalid inputs for type; qed");
    Origin::from(RawOrigin::Signed(zero_account_id))
  }
}

parameter_types! {
  pub static MockBlockNumberProvider: u64 = 0;
  pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
}

impl BlockNumberProvider for MockBlockNumberProvider {
  type BlockNumber = u64;

  fn current_block_number() -> Self::BlockNumber {
    Self::get()
  }
}

impl Config for Runtime {
  type Event = Event;
  type Currency = PalletBalances;
  type MinVestedTransfer = ConstU128<5>;
  type VestedTransferOrigin = EnsureAliceOrBob;
  type WeightInfo = ();
  type MaxVestingSchedules = ConstU32<2>;
  type BlockNumberProvider = MockBlockNumberProvider;
  type CurrencyTidefi = Adapter<AccountId>;
  type TreasuryPalletId = TreasuryPalletId;
  type ForceOrigin = EnsureRoot<Self::AccountId>;
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
      CurrencyId::Tdfy => PalletBalances::total_issuance(),
      CurrencyId::Wrapped(_) => unimplemented!(),
    }
  }

  fn balance(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
    match asset {
      CurrencyId::Tdfy => PalletBalances::balance(who),
      CurrencyId::Wrapped(_) => unimplemented!(),
    }
  }

  fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
    match asset {
      CurrencyId::Tdfy => {
        <pallet_balances::Pallet<mock::Runtime> as frame_support::traits::fungible::Inspect<
          AccountId,
        >>::minimum_balance()
      }
      CurrencyId::Wrapped(_) => unimplemented!(),
    }
  }

  fn reducible_balance(asset: Self::AssetId, who: &AccountId, keep_alive: bool) -> Self::Balance {
    match asset {
      CurrencyId::Tdfy => PalletBalances::reducible_balance(who, keep_alive),
      CurrencyId::Wrapped(_) => unimplemented!(),
    }
  }

  fn can_deposit(
    asset: Self::AssetId,
    who: &AccountId,
    amount: Self::Balance,
    mint: bool,
  ) -> DepositConsequence {
    match asset {
      CurrencyId::Tdfy => PalletBalances::can_deposit(who, amount, mint),
      CurrencyId::Wrapped(_) => unimplemented!(),
    }
  }

  fn can_withdraw(
    asset: Self::AssetId,
    who: &AccountId,
    amount: Self::Balance,
  ) -> WithdrawConsequence<Self::Balance> {
    match asset {
      CurrencyId::Tdfy => PalletBalances::can_withdraw(who, amount),
      CurrencyId::Wrapped(_) => unimplemented!(),
    }
  }
}

impl Mutate<AccountId> for Adapter<AccountId> {
  fn mint_into(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
    match asset {
      CurrencyId::Tdfy => PalletBalances::mint_into(who, amount),
      CurrencyId::Wrapped(_) => unimplemented!(),
    }
  }

  fn burn_from(
    asset: Self::AssetId,
    who: &AccountId,
    amount: Balance,
  ) -> Result<Balance, DispatchError> {
    match asset {
      CurrencyId::Tdfy => PalletBalances::burn_from(who, amount),
      CurrencyId::Wrapped(_) => unimplemented!(),
    }
  }
}

impl Transfer<AccountId> for Adapter<AccountId>
{
  fn transfer(
    asset: Self::AssetId,
    source: &AccountId,
    dest: &AccountId,
    amount: Self::Balance,
    keep_alive: bool,
  ) -> Result<Balance, DispatchError> {
    match asset {
      CurrencyId::Tdfy => {
        <PalletBalances as FungibleTransfer<AccountId>>::transfer(source, dest, amount, keep_alive)
      }
      CurrencyId::Wrapped(_) => unimplemented!()
    }
  }
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
