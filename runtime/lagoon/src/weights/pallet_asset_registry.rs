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

//! Autogenerated weights for `pallet_asset_registry`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-05, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --pallet=*
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

/// Weight functions for `pallet_asset_registry`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_asset_registry::WeightInfo for WeightInfo<T> {
	/// Storage: AssetRegistry AssetRegistryAccountId (r:1 w:0)
	/// Proof: AssetRegistry AssetRegistryAccountId (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	fn set_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `136`
		//  Estimated: `527`
		// Minimum execution time: 15_892 nanoseconds.
		Weight::from_parts(17_894_000, 527)
			.saturating_add(T::DbWeight::get().reads(1))
	}
}
