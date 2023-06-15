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
//! DATE: 2023-02-21, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
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
	/// Storage: Fees ActiveEra (r:1 w:1)
	/// Proof: Fees ActiveEra (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	/// Storage: Security CurrentBlockCount (r:1 w:0)
	/// Proof: Security CurrentBlockCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Fees CurrentSession (r:1 w:1)
	/// Proof: Fees CurrentSession (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Fees SessionTotalFees (r:7 w:0)
	/// Proof: Fees SessionTotalFees (max_values: None, max_size: Some(93), added: 2568, mode: MaxEncodedLen)
	/// Storage: Assets Asset (r:6 w:6)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:12 w:12)
	/// Proof: Assets Account (max_values: Some(300000), max_size: Some(118), added: 2593, mode: MaxEncodedLen)
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: TidefiStaking PendingStoredSessions (r:1 w:1)
	/// Proof: TidefiStaking PendingStoredSessions (max_values: None, max_size: Some(235), added: 2710, mode: MaxEncodedLen)
	/// Storage: TidefiStaking CounterForPendingStoredSessions (r:1 w:1)
	/// Proof: TidefiStaking CounterForPendingStoredSessions (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: TidefiStaking AccountStakes (r:10001 w:0)
	/// Proof: TidefiStaking AccountStakes (max_values: None, max_size: Some(949), added: 3424, mode: MaxEncodedLen)
	/// Storage: TidefiStaking QueueCompound (r:10000 w:10000)
	/// Proof: TidefiStaking QueueCompound (max_values: None, max_size: Some(56), added: 2531, mode: MaxEncodedLen)
	/// Storage: TidefiStaking CounterForQueueCompound (r:1 w:1)
	/// Proof: TidefiStaking CounterForQueueCompound (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Fees StoredSessions (r:2 w:1)
	/// Proof: Fees StoredSessions (max_values: None, max_size: Some(24), added: 2499, mode: MaxEncodedLen)
	/// Storage: TidefiStaking InterestCompoundLastSession (r:0 w:1)
	/// Proof: TidefiStaking InterestCompoundLastSession (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: TidefiStaking SessionTotalFees (r:0 w:6)
	/// Proof: TidefiStaking SessionTotalFees (max_values: None, max_size: Some(61), added: 2536, mode: MaxEncodedLen)
	/// The range of component `a` is `[1, 10000]`.
	/// The range of component `b` is `[1, 6]`.
	fn on_finalize(a: u32, b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `4520 + a * (85 ±0) + b * (435 ±0)`
		//  Estimated: `21433 + b * (10439 ±0) + a * (5955 ±0)`
		// Minimum execution time: 420_768 nanoseconds.
		Weight::from_parts(460_044_000, 21433)
			// Standard Error: 9_851
			.saturating_add(Weight::from_ref_time(12_535_319).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(12))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().reads((4_u64).saturating_mul(b.into())))
			.saturating_add(T::DbWeight::get().writes(9))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes((4_u64).saturating_mul(b.into())))
			.saturating_add(Weight::from_proof_size(10439).saturating_mul(b.into()))
			.saturating_add(Weight::from_proof_size(5955).saturating_mul(a.into()))
	}
}
