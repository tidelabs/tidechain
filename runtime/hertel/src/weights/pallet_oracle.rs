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

//! Autogenerated weights for `pallet_oracle`
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
// --pallet=pallet_oracle
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --header=./file_header.txt
// --output=./runtime/hertel/src/weights/pallet_oracle.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_oracle`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_oracle::WeightInfo for WeightInfo<T> {
	// Storage: WraprOracle OracleAccountId (r:1 w:0)
	// Storage: WraprOracle OracleStatus (r:0 w:1)
	fn set_status() -> Weight {
		(28_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: WraprOracle OracleAccountId (r:1 w:1)
	fn set_account_id() -> Weight {
		(30_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: WraprSecurity ChainStatus (r:1 w:0)
	// Storage: WraprOracle OracleStatus (r:1 w:0)
	// Storage: WraprOracle OracleAccountId (r:1 w:0)
	// Storage: WraprOracle Trades (r:2 w:2)
	// Storage: Assets Asset (r:2 w:2)
	// Storage: Assets Account (r:6 w:6)
	// Storage: WraprFees FeePercentageAmount (r:1 w:0)
	// Storage: System Account (r:3 w:3)
	// Storage: WraprFees ActiveEra (r:1 w:0)
	// Storage: WraprFees EraTotalFees (r:2 w:2)
	// Storage: WraprFees AccountFees (r:2 w:2)
	fn confirm_trade() -> Weight {
		(377_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(22 as Weight))
			.saturating_add(T::DbWeight::get().writes(17 as Weight))
	}
}
