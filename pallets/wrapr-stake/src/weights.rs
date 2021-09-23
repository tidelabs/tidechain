#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_wrapr.
pub trait WeightInfo {
   fn stake() -> Weight;
}

/// Weights for pallet_wrapr using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {

   fn stake() -> Weight {
      14_100_000_u64
      .saturating_add(T::DbWeight::get().reads(6_u64))
      .saturating_add(T::DbWeight::get().writes(5_u64))
   }
}
