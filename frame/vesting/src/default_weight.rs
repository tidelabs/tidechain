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

//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

impl crate::WeightInfo for () {
  fn vested_transfer() -> Weight {
    (Weight::from_ref_time(310_862_000))
      .saturating_add(DbWeight::get().reads(4_u64))
      .saturating_add(DbWeight::get().writes(4_u64))
  }
  fn claim(i: u32) -> Weight {
    (Weight::from_ref_time(158_614_000))
      .saturating_add(Weight::from_ref_time(958_000).saturating_mul(i.into()))
      .saturating_add(DbWeight::get().reads(3_u64))
      .saturating_add(DbWeight::get().writes(3_u64))
  }
  fn update_vesting_schedules(i: u32) -> Weight {
    (Weight::from_ref_time(119_811_000))
      .saturating_add(Weight::from_ref_time(2_320_000).saturating_mul(i.into()))
      .saturating_add(DbWeight::get().reads(2_u64))
      .saturating_add(DbWeight::get().writes(3_u64))
  }
}
