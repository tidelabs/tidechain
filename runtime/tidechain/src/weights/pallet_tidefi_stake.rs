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

//! Autogenerated weights for `pallet_tidefi_stake`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-22, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `ns3206536`, CPU: `Intel(R) Xeon(R) D-2141I CPU @ 2.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("tidechain-dev"), DB CACHE: 1024

// Executed Command:
// target/production/tidechain
// benchmark
// pallet
// --chain=tidechain-dev
// --steps=50
// --repeat=20
// --pallet=pallet_tidefi_stake
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

/// Weight functions for `pallet_tidefi_stake`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_tidefi_stake::WeightInfo for WeightInfo<T> {
	// Storage: TidefiStaking OperatorAccountId (r:0 w:1)
	fn set_operator_account_id() -> Weight {
		(18_313_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: TidefiStaking StakingPeriodRewards (r:1 w:0)
	// Storage: TidefiStaking StakingCurrencyMeta (r:1 w:0)
	// Storage: Security Nonce (r:1 w:1)
	// Storage: System ParentHash (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: TidefiStaking StakingPool (r:1 w:1)
	// Storage: Security CurrentBlockCount (r:1 w:0)
	// Storage: TidefiStaking AccountStakes (r:1 w:1)
	// Storage: TidefiStaking InterestCompoundLastSession (r:1 w:0)
	fn stake() -> Weight {
		(107_309_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(11 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: TidefiStaking AccountStakes (r:1 w:1)
	// Storage: Security CurrentBlockCount (r:1 w:0)
	// Storage: TidefiStaking UnstakeFee (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: TidefiStaking QueueUnstake (r:1 w:1)
	// Storage: TidefiStaking CounterForQueueUnstake (r:1 w:1)
	// Storage: TidefiStaking OperatorAccountId (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	fn unstake() -> Weight {
		(95_587_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(10 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	// Storage: TidefiStaking CounterForQueueUnstake (r:1 w:0)
	// Storage: TidefiStaking CounterForQueueCompound (r:1 w:0)
	// Storage: TidefiStaking CounterForPendingStoredSessions (r:1 w:1)
	// Storage: TidefiStaking QueueCompound (r:1 w:0)
	// Storage: TidefiStaking PendingStoredSessions (r:2 w:1)
	// Storage: TidefiStaking SessionTotalFees (r:2 w:0)
	// Storage: TidefiStaking OperatorAccountId (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	/// The range of component `b` is `[1, 500]`.
	fn on_idle_compound_finalize(b: u32, ) -> Weight {
		(11_101_000 as Weight)
			// Standard Error: 38_000
			.saturating_add((28_727_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().reads(10 as Weight))
			.saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(b as Weight)))
			.saturating_add(T::DbWeight::get().writes(9 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
	}
	// Storage: TidefiStaking CounterForQueueUnstake (r:1 w:0)
	// Storage: TidefiStaking CounterForQueueCompound (r:1 w:1)
	// Storage: TidefiStaking CounterForPendingStoredSessions (r:1 w:0)
	// Storage: TidefiStaking QueueCompound (r:2 w:1)
	// Storage: TidefiStaking PendingStoredSessions (r:2 w:1)
	// Storage: TidefiStaking SessionTotalFees (r:2 w:0)
	// Storage: TidefiStaking AccountStakes (r:1 w:1)
	// Storage: Security CurrentBlockCount (r:1 w:0)
	// Storage: TidefiStaking StakingPeriodRewards (r:1 w:0)
	// Storage: TidefiStaking StakingPool (r:1 w:1)
	/// The range of component `b` is `[1, 500]`.
	fn on_idle_compound(b: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 30_000
			.saturating_add((26_402_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().reads(11 as Weight))
			.saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(b as Weight)))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
			.saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(b as Weight)))
	}
	// Storage: TidefiStaking CounterForQueueUnstake (r:1 w:1)
	// Storage: TidefiStaking CounterForQueueCompound (r:1 w:0)
	// Storage: TidefiStaking CounterForPendingStoredSessions (r:1 w:0)
	// Storage: Security CurrentBlockCount (r:1 w:0)
	// Storage: TidefiStaking QueueUnstake (r:2 w:1)
	// Storage: TidefiStaking AccountStakes (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: TidefiStaking StakingPool (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: TidefiStaking CounterForAccountStakes (r:1 w:1)
	/// The range of component `b` is `[1, 500]`.
	fn on_idle_unstake(b: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 40_000
			.saturating_add((58_257_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().reads(10 as Weight))
			.saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(b as Weight)))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
			.saturating_add(T::DbWeight::get().writes((3 as Weight).saturating_mul(b as Weight)))
	}
}
