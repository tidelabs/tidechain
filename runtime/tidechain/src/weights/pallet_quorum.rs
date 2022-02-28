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

//! Autogenerated weights for `pallet_quorum`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-02-17, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("hertel-dev"), DB CACHE: 128

// Executed Command:
// target/release/tidechain
// benchmark
// --chain=hertel-dev
// --steps=50
// --repeat=20
// --pallet=*
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --header=./file_header.txt
// --output=./runtime/hertel/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_quorum`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_quorum::WeightInfo for WeightInfo<T> {
    fn submit_proposal() -> Weight {
        61_000_300_u64
            .saturating_add(T::DbWeight::get().reads(6_u64))
            .saturating_add(T::DbWeight::get().writes(5_u64))
    }
    fn acknowledge_proposal() -> Weight {
        62_000_300_u64
         .saturating_add(T::DbWeight::get().reads(6_u64))
         .saturating_add(T::DbWeight::get().writes(5_u64))
     }
     fn reject_proposal() -> Weight {
        63_000_400_u64
           .saturating_add(T::DbWeight::get().reads(6_u64))
           .saturating_add(T::DbWeight::get().writes(5_u64))
     }
     fn eval_proposal_state() -> Weight {
        64_000_400_u64
           .saturating_add(T::DbWeight::get().reads(6_u64))
           .saturating_add(T::DbWeight::get().writes(5_u64))
     }
     fn submit_public_keys(_k: u32) -> Weight {
        65_000_400_u64
           .saturating_add(T::DbWeight::get().reads(6_u64))
           .saturating_add(T::DbWeight::get().writes(5_u64))
     } 
}
