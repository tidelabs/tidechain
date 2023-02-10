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
}

/// Weights for `pallet_tidefi` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {

   fn stake() -> Weight {
      14_100_000_u64
      .saturating_add(T::DbWeight::get().reads(6_u64))
      .saturating_add(T::DbWeight::get().writes(5_u64))
   }
   fn unstake() -> Weight {
      18_100_000_u64
      .saturating_add(T::DbWeight::get().reads(6_u64))
      .saturating_add(T::DbWeight::get().writes(5_u64))
   }
	fn on_idle_compound(b: u32) -> Weight {
		// Minimum execution time: 1_775_293 nanoseconds.
		1_787_133_000_u64
			// Standard Error: 17_109_142
			.saturating_add(1_787_133_000_u64.saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(1))
	}  
	fn on_idle_compound_finalize(b: u32) -> Weight {
		// Minimum execution time: 1_775_293 nanoseconds.
		1_787_133_000_u64
			// Standard Error: 17_109_142
			.saturating_add(1_787_133_000_u64.saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(1))
	}   
	fn on_idle_unstake(b: u32) -> Weight {
		// Minimum execution time: 1_775_293 nanoseconds.
		1_787_133_000_u64
			// Standard Error: 17_109_142
			.saturating_add(1_787_133_000_u64.saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(1))
	}   
}
