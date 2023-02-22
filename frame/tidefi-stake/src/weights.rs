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

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for `pallet_tidefi`.
pub trait WeightInfo {
   fn stake() -> Weight;
   fn unstake() -> Weight;
   fn on_idle_compound(b: u32) -> Weight;
   fn on_idle_compound_finalize(b: u32) -> Weight;
   fn on_idle_unstake(b: u32) -> Weight;
   fn set_operator_account_id() -> Weight;
}

/// Weights for `pallet_tidefi` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn stake() -> Weight {
		(106_968_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(11 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}

	fn unstake() -> Weight {
		(87_151_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(9 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}

	fn on_idle_compound_finalize(b: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 32_000
			.saturating_add((30_468_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(b as Weight)))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
	}

	fn on_idle_compound(b: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 28_000
			.saturating_add((27_581_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().reads(10 as Weight))
			.saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(b as Weight)))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
			.saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(b as Weight)))
	}

	fn on_idle_unstake(b: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 39_000
			.saturating_add((59_405_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(b as Weight)))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
			.saturating_add(T::DbWeight::get().writes((3 as Weight).saturating_mul(b as Weight)))
	}

	fn set_operator_account_id() -> Weight {
		(18_554_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
