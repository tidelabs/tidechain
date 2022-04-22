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
//! DATE: 2022-04-22, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("lagoon-dev"), DB CACHE: 1024

// Executed Command:
// target/production/tidechain
// benchmark
// pallet
// --chain=lagoon-dev
// --steps=50
// --repeat=20
// --pallet=pallet_oracle
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --header=./FILE_TEMPLATE
// --output=./runtime/lagoon/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_oracle`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_oracle::WeightInfo for WeightInfo<T> {
	// Storage: Oracle OracleAccountId (r:1 w:0)
	// Storage: Security CurrentBlockCount (r:1 w:0)
	// Storage: Oracle LastSeen (r:0 w:1)
	// Storage: Oracle OracleStatus (r:0 w:1)
	fn set_status() -> Weight {
		(21_148_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Oracle OracleAccountId (r:1 w:1)
	// Storage: Security CurrentBlockCount (r:1 w:0)
	// Storage: Oracle LastSeen (r:0 w:1)
	fn set_account_id() -> Weight {
		(21_932_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Oracle OracleAccountId (r:1 w:0)
	// Storage: Security CurrentBlockCount (r:1 w:0)
	// Storage: Oracle MarketMakers (r:0 w:1)
	// Storage: Oracle LastSeen (r:0 w:1)
	fn add_market_maker() -> Weight {
		(23_032_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Oracle OracleAccountId (r:1 w:0)
	// Storage: Security CurrentBlockCount (r:1 w:0)
	// Storage: Oracle LastSeen (r:0 w:1)
	fn im_alive() -> Weight {
		(8_032_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Oracle OracleAccountId (r:1 w:0)
	// Storage: Security CurrentBlockCount (r:1 w:0)
	// Storage: Oracle MarketMakers (r:0 w:1)
	// Storage: Oracle LastSeen (r:0 w:1)
	fn remove_market_maker() -> Weight {
		(21_422_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Security ChainStatus (r:1 w:0)
	// Storage: Oracle OracleStatus (r:1 w:0)
	// Storage: Oracle OracleAccountId (r:1 w:0)
	// Storage: Oracle Swaps (r:2 w:2)
	// Storage: Assets Account (r:6 w:6)
	// Storage: Assets Asset (r:2 w:2)
	// Storage: Fees OrderBookPrice (r:4 w:0)
	// Storage: System Account (r:3 w:3)
	// Storage: Fees ActiveEra (r:1 w:0)
	// Storage: Fees CurrentSession (r:1 w:0)
	// Storage: Fees SunrisePools (r:1 w:1)
	// Storage: Fees SunriseRewards (r:2 w:2)
	// Storage: Fees EraTotalFees (r:2 w:2)
	// Storage: Fees SessionTotalFees (r:2 w:2)
	// Storage: Fees AccountFees (r:2 w:2)
	// Storage: Oracle AccountSwaps (r:2 w:2)
	// Storage: Security CurrentBlockCount (r:1 w:0)
	// Storage: Oracle LastSeen (r:0 w:1)
	fn confirm_swap() -> Weight {
		(330_886_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(34 as Weight))
			.saturating_add(T::DbWeight::get().writes(25 as Weight))
	}
}
