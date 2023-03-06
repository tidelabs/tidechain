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

//! Autogenerated weights for `pallet_fees`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-13, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `ns3206536`, CPU: `Intel(R) Xeon(R) D-2141I CPU @ 2.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("lagoon-dev"), DB CACHE: 1024

// Executed Command:
// target/production/tidechain
// benchmark
// pallet
// --chain=lagoon-dev
// --steps=50
// --repeat=20
// --pallet=pallet_fees
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

/// Weight functions for `pallet_fees`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_fees::WeightInfo for WeightInfo<T> {
	// Storage: Fees ActiveEra (r:1 w:1)
	// Storage: Security CurrentBlockCount (r:1 w:0)
	// Storage: Fees CurrentSession (r:1 w:1)
	// Storage: Fees SessionTotalFees (r:7 w:0)
	// Storage: Assets Asset (r:6 w:6)
	// Storage: Assets Account (r:12 w:12)
	// Storage: System Account (r:2 w:2)
	// Storage: TidefiStaking PendingStoredSessions (r:1 w:1)
	// Storage: TidefiStaking CounterForPendingStoredSessions (r:1 w:1)
	// Storage: TidefiStaking AccountStakes (r:2 w:0)
	// Storage: TidefiStaking QueueCompound (r:1 w:1)
	// Storage: TidefiStaking CounterForQueueCompound (r:1 w:1)
	// Storage: Fees StoredSessions (r:2 w:1)
	// Storage: TidefiStaking InterestCompoundLastSession (r:0 w:1)
	// Storage: TidefiStaking SessionTotalFees (r:0 w:6)
	/// The range of component `a` is `[1, 10000]`.
	/// The range of component `b` is `[2, 6]`.
	/// The range of component `a` is `[1, 10000]`.
	/// The range of component `b` is `[2, 6]`.
	fn on_finalize(a: u32, b: u32) -> Weight {
		Weight::from_ref_time(0)
			// Standard Error: 20_000
			.saturating_add(Weight::from_ref_time(14_074_000).saturating_mul(a.into()))
			// Standard Error: 89_132_000
			.saturating_add(Weight::from_ref_time(45_319_000).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(12_u64))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().reads((4_u64).saturating_mul(b.into())))
			.saturating_add(T::DbWeight::get().writes(9_u64))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes((4_u64).saturating_mul(b.into())))
	}
}
