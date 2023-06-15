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
		Weight::from_ref_time(106_968_000)
			.saturating_add(T::DbWeight::get().reads(11))
			.saturating_add(T::DbWeight::get().writes(6))
	}

	fn unstake() -> Weight {
		Weight::from_ref_time(87_151_000)
			.saturating_add(T::DbWeight::get().reads(9 ))
			.saturating_add(T::DbWeight::get().writes(6 ))
	}

	fn on_idle_compound_finalize(b: u32, ) -> Weight {
		Weight::from_ref_time(0)
			// Standard Error: 32_000
			.saturating_add(Weight::from_ref_time(30_468_000).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(8_u64))
			.saturating_add(T::DbWeight::get().reads(1_u64.saturating_mul(b.into())))
			.saturating_add(T::DbWeight::get().writes(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64.saturating_mul(b.into())))
	}

	fn on_idle_compound(b: u32, ) -> Weight {
		Weight::from_ref_time(0)
			// Standard Error: 28_000
			.saturating_add(Weight::from_ref_time(27_581_000).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(10_u64))
			.saturating_add(T::DbWeight::get().reads(2_u64.saturating_mul(b.into())))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64.saturating_mul(b.into())))
	}

	fn on_idle_unstake(b: u32, ) -> Weight {
		Weight::from_ref_time(0)
			// Standard Error: 39_000
			.saturating_add(Weight::from_ref_time(59_405_000).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(8_u64))
			.saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(b.into())))
			.saturating_add(T::DbWeight::get().writes(5_u64))
			.saturating_add(T::DbWeight::get().writes((3_u64).saturating_mul(b.into())))
	}
}
