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
   fn submit_proposal() -> Weight;
   fn acknowledge_proposal() -> Weight;
   fn acknowledge_burned() -> Weight;
   fn reject_proposal() -> Weight;
   fn eval_proposal_state() -> Weight;
   fn submit_public_keys(_k: u32) -> Weight;
}

/// Weights for `pallet_tidefi` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
   fn submit_proposal() -> Weight {
      Weight::from_ref_time(61_000_300)
          .saturating_add(T::DbWeight::get().reads(6_u64))
          .saturating_add(T::DbWeight::get().writes(5_u64))
  }
  fn acknowledge_proposal() -> Weight {
      Weight::from_ref_time(62_000_300)
         .saturating_add(T::DbWeight::get().reads(6_u64))
         .saturating_add(T::DbWeight::get().writes(5_u64))
   }
   fn acknowledge_burned() -> Weight {
      Weight::from_ref_time(60_000_300)
         .saturating_add(T::DbWeight::get().reads(6_u64))
         .saturating_add(T::DbWeight::get().writes(5_u64))
   }
   fn reject_proposal() -> Weight {
      Weight::from_ref_time(63_000_300)
         .saturating_add(T::DbWeight::get().reads(6_u64))
         .saturating_add(T::DbWeight::get().writes(5_u64))
   }
   fn eval_proposal_state() -> Weight {
      Weight::from_ref_time(64_000_300)
         .saturating_add(T::DbWeight::get().reads(6_u64))
         .saturating_add(T::DbWeight::get().writes(5_u64))
   }
   fn submit_public_keys(k: u32) -> Weight {
      Weight::from_ref_time(65_000_300)
         .saturating_add(Weight::from_ref_time(200_000).saturating_mul(k as u64))
         .saturating_add(T::DbWeight::get().reads(6_u64))
         .saturating_add(T::DbWeight::get().writes(5_u64))
   }
}
