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

//! Autogenerated weights for `frame_election_provider_support`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-09-12, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `ns3206536`, CPU: `Intel(R) Xeon(R) D-2141I CPU @ 2.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("tidechain-dev"), DB CACHE: 1024

// Executed Command:
// target/production/tidechain
// benchmark
// pallet
// --chain=tidechain-dev
// --steps=50
// --repeat=20
// --pallet=*
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --header=./FILE_TEMPLATE
// --output=./runtime/tidechain/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `frame_election_provider_support`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> frame_election_provider_support::WeightInfo for WeightInfo<T> {
	/// The range of component `v` is `[1000, 2000]`.
	/// The range of component `t` is `[500, 1000]`.
	/// The range of component `d` is `[5, 16]`.
	fn phragmen(v: u32, _t: u32, d: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 98_000
			.saturating_add((35_265_000 as Weight).saturating_mul(v as Weight))
			// Standard Error: 13_707_000
			.saturating_add((5_729_241_000 as Weight).saturating_mul(d as Weight))
	}
	/// The range of component `v` is `[1000, 2000]`.
	/// The range of component `t` is `[500, 1000]`.
	/// The range of component `d` is `[5, 16]`.
	fn phragmms(v: u32, _t: u32, d: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 89_000
			.saturating_add((24_849_000 as Weight).saturating_mul(v as Weight))
			// Standard Error: 12_449_000
			.saturating_add((4_245_734_000 as Weight).saturating_mul(d as Weight))
	}
}
