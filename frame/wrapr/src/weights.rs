#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
   fn request_withdrawal() -> Weight;
   fn request_trade() -> Weight;
   fn request_stake() -> Weight;
   fn transfer() -> Weight;
}

/// Weights for `pallet_wrapr` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
   fn request_withdrawal() -> Weight {
      14_200_000_u64
      .saturating_add(T::DbWeight::get().reads(6_u64))
      .saturating_add(T::DbWeight::get().writes(5_u64))
   }
   fn request_trade() -> Weight {
      14_300_000_u64
      .saturating_add(T::DbWeight::get().reads(6_u64))
      .saturating_add(T::DbWeight::get().writes(5_u64))
   }
   fn request_stake() -> Weight {
      14_400_000_u64
      .saturating_add(T::DbWeight::get().reads(6_u64))
      .saturating_add(T::DbWeight::get().writes(5_u64))
   }
   fn transfer() -> Weight {
      18_400_000_u64
      .saturating_add(T::DbWeight::get().reads(6_u64))
      .saturating_add(T::DbWeight::get().writes(5_u64))
   }
}
