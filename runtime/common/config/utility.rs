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
  constants::currency::deposit, types::Balance, Balances, Call, Event, OriginCaller, Runtime,
};
use frame_support::parameter_types;

impl pallet_utility::Config for Runtime {
  type PalletsOrigin = OriginCaller;
  type Event = Event;
  type Call = Call;
  type WeightInfo = crate::weights::pallet_utility::WeightInfo<Runtime>;
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
  type WeightInfo = crate::weights::pallet_multisig::WeightInfo<Runtime>;
}
