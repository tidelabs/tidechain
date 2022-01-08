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
//! DATE: 2022-01-06, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("hertel-dev"), DB CACHE: 128

// Executed Command:
// ./target/release/tidechain
// benchmark
// --chain=hertel-dev
// --steps=50
// --repeat=20
// --pallet=pallet_quorum
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --header=./file_header.txt
// --output=./runtime/hertel/src/weights/pallet_quorum.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_quorum`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_quorum::WeightInfo for WeightInfo<T> {
	// Storage: WraprQuorum QuorumAccountId (r:1 w:0)
	// Storage: WraprQuorum QuorumStatus (r:0 w:1)
	fn set_status() -> Weight {
		(22_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: WraprQuorum QuorumAccountId (r:1 w:1)
	fn set_account_id() -> Weight {
		(23_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: WraprSecurity ChainStatus (r:1 w:0)
	// Storage: WraprQuorum QuorumStatus (r:1 w:0)
	// Storage: WraprQuorum QuorumAccountId (r:1 w:0)
	// Storage: WraprQuorum Withdrawals (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	fn confirm_withdrawal() -> Weight {
		(68_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: WraprSecurity ChainStatus (r:1 w:0)
	// Storage: WraprQuorum QuorumStatus (r:1 w:0)
	// Storage: WraprQuorum QuorumAccountId (r:1 w:0)
	fn mint() -> Weight {
		(63_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
	}
}
