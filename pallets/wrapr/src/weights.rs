#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_wrapr.
pub trait WeightInfo {
   fn request_withdrawal() -> Weight;
}

/// Weights for pallet_wrapr using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
   fn request_withdrawal() -> Weight {
         (67_000_000 as Weight)
         .saturating_add(T::DbWeight::get().reads(6 as Weight))
         .saturating_add(T::DbWeight::get().writes(5 as Weight))
   }
}
