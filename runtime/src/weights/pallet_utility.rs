
//! Autogenerated weights for `pallet_utility`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2021-09-11, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// ./target/release/tidefi-node
// benchmark
// --chain
// dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet=pallet_utility
// --extrinsic=*
// --steps
// 50
// --repeat
// 20
// --output=benchout/pallet_utility.rs


#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_utility.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_utility::WeightInfo for WeightInfo<T> {
	fn batch(c: u32, ) -> Weight {
		(21_521_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((4_684_000 as Weight).saturating_mul(c as Weight))
	}
	fn as_derivative() -> Weight {
		(2_643_000 as Weight)
	}
	fn batch_all(c: u32, ) -> Weight {
		(18_487_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((5_067_000 as Weight).saturating_mul(c as Weight))
	}
}