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
  constants::currency::{deposit, TDFY},
  types::{AccountId, Balance, BlockNumber},
  Balances, OriginCaller, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, System,
  TreasuryPalletId,
};
use frame_support::{parameter_types, traits::EnsureOrigin};
use frame_system::RawOrigin;
use sp_runtime::traits::{AccountIdConversion, BlockNumberProvider};

impl pallet_utility::Config for Runtime {
  type PalletsOrigin = OriginCaller;
  type RuntimeEvent = RuntimeEvent;
  type RuntimeCall = RuntimeCall;
  type WeightInfo = crate::weights::pallet_utility::WeightInfo<Runtime>;
}

parameter_types! {
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const DepositBase: Balance = deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);
    pub const MaxSignatories: u16 = 100;

    pub MinVestedTransfer: Balance = TDFY;
    pub const MaxVestingSchedules: u32 = 300;

}

impl pallet_multisig::Config for Runtime {
  type RuntimeEvent = RuntimeEvent;
  type RuntimeCall = RuntimeCall;
  type Currency = Balances;
  type DepositBase = DepositBase;
  type DepositFactor = DepositFactor;
  type MaxSignatories = MaxSignatories;
  type WeightInfo = crate::weights::pallet_multisig::WeightInfo<Runtime>;
}

pub struct EnsureRootOrTreasury;
impl EnsureOrigin<RuntimeOrigin> for EnsureRootOrTreasury {
  type Success = AccountId;

  fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
    Into::<Result<RawOrigin<AccountId>, RuntimeOrigin>>::into(o).and_then(|o| match o {
      RawOrigin::Root => Ok(TreasuryPalletId::get().into_account_truncating()),
      RawOrigin::Signed(caller) => {
        if caller == TreasuryPalletId::get().into_account_truncating() {
          Ok(caller)
        } else {
          Err(RuntimeOrigin::from(Some(caller)))
        }
      }
      r => Err(RuntimeOrigin::from(r)),
    })
  }

  #[cfg(feature = "runtime-benchmarks")]
  fn successful_origin() -> RuntimeOrigin {
    RuntimeOrigin::from(RawOrigin::Signed(
      TreasuryPalletId::get().into_account_truncating(),
    ))
  }
}

pub struct SusbtrateBlockNumberProvider;
impl BlockNumberProvider for SusbtrateBlockNumberProvider {
  type BlockNumber = BlockNumber;

  fn current_block_number() -> Self::BlockNumber {
    System::block_number()
  }
}

impl pallet_vesting::Config for Runtime {
  type RuntimeEvent = RuntimeEvent;
  type Currency = Balances;
  type MinVestedTransfer = MinVestedTransfer;
  type VestedTransferOrigin = EnsureRootOrTreasury;
  type WeightInfo = pallet_vesting::SubstrateWeight<Runtime>;
  type MaxVestingSchedules = MaxVestingSchedules;
  type BlockNumberProvider = SusbtrateBlockNumberProvider;
}
