#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for `pallet_tidefi`.
pub trait WeightInfo {
   fn submit_proposal() -> Weight;
   fn acknowledge_proposal() -> Weight;
   fn reject_proposal() -> Weight;
   fn eval_proposal_state() -> Weight;
   fn submit_public_keys(_k: u32) -> Weight;
}

/// Weights for `pallet_tidefi` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
   fn submit_proposal() -> Weight {
      61_000_300_u64
          .saturating_add(T::DbWeight::get().reads(6_u64))
          .saturating_add(T::DbWeight::get().writes(5_u64))
  }
  fn acknowledge_proposal() -> Weight {
      62_000_300_u64
       .saturating_add(T::DbWeight::get().reads(6_u64))
       .saturating_add(T::DbWeight::get().writes(5_u64))
   }
   fn reject_proposal() -> Weight {
      63_000_400_u64
         .saturating_add(T::DbWeight::get().reads(6_u64))
         .saturating_add(T::DbWeight::get().writes(5_u64))
   }
   fn eval_proposal_state() -> Weight {
      64_000_400_u64
         .saturating_add(T::DbWeight::get().reads(6_u64))
         .saturating_add(T::DbWeight::get().writes(5_u64))
   }
   fn submit_public_keys(k: u32) -> Weight {
      65_000_400_u64
         .saturating_add((2_000 as Weight).saturating_mul(k as Weight))
         .saturating_add(T::DbWeight::get().reads(6_u64))
         .saturating_add(T::DbWeight::get().writes(5_u64))
   }
}
