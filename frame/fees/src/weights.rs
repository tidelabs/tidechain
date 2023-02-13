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

/// Weight functions needed for `pallet_fees`.
pub trait WeightInfo {
   fn on_finalize(a: u32, b: u32) -> Weight;
}

/// Weights for `pallet_fees` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn on_finalize(a: u32, b: u32) -> Weight {
		(0 as Weight)
			// Standard Error: 20_000
			.saturating_add((14_074_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 89_132_000
			.saturating_add((45_319_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().reads(12 as Weight))
			.saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
			.saturating_add(T::DbWeight::get().reads((4 as Weight).saturating_mul(b as Weight)))
			.saturating_add(T::DbWeight::get().writes(9 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
			.saturating_add(T::DbWeight::get().writes((4 as Weight).saturating_mul(b as Weight)))
	}

}
