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
  constants::currency::deposit,
  types::{Balance, BlakeTwo256},
  Balances, Call, Event, Runtime,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{parameter_types, traits::InstanceFilter, RuntimeDebug};
use scale_info::TypeInfo;

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
  Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo, RuntimeDebug, MaxEncodedLen,
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
        Call::Balances(..) | Call::Indices(pallet_indices::Call::transfer { .. })
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
  type MaxPending = MaxPending;
  type CallHasher = BlakeTwo256;
  type AnnouncementDepositBase = AnnouncementDepositBase;
  type AnnouncementDepositFactor = AnnouncementDepositFactor;
  type WeightInfo = crate::weights::pallet_proxy::WeightInfo<Runtime>;
}
