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

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
#![allow(clippy::from_over_into)]
#![allow(clippy::manual_range_contains)]

use frame_support::{construct_runtime, parameter_types, weights::Weight, PalletId};
use frame_system::limits::{BlockLength, BlockWeights};
use sp_runtime::traits::Get;

use pallet_session::historical as pallet_session_historical;

#[cfg(any(feature = "std", test))]
pub use frame_system::Call as SystemCall;
#[cfg(any(feature = "std", test))]
pub use pallet_balances::Call as BalancesCall;
#[cfg(any(feature = "std", test))]
pub use pallet_staking::StakerStatus;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

use sp_runtime::{create_runtime_str, impl_opaque_keys, DispatchError, DispatchResult};
use sp_std::prelude::*;
#[cfg(any(feature = "std", test))]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

#[cfg(feature = "std")]
pub use crate::api::{api::dispatch, RuntimeApi};
pub use crate::types::{
  AccountId, AccountIndex, Address, AssetId, Balance, Block, BlockId, BlockNumber,
  CheckedExtrinsic, Hash, Header, Moment, Nonce, Signature, SignedBlock, SignedExtra,
  SignedPayload, UncheckedExtrinsic, BABE_GENESIS_EPOCH_CONFIG,
};

mod config;

/// Implementations of some helper traits passed into runtime modules as associated types.
#[path = "../../common/impls.rs"]
mod impls;

/// Constant values used within the runtime.
#[path = "../../common/constants.rs"]
pub mod constants;

#[path = "../../common/types.rs"]
pub mod types;

#[path = "../../common/api.rs"]
mod api;

mod weights;

// Voter bag threshold definitions.
mod bag_thresholds;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// Tidefi Prefix
pub const SS58_PREFIX: u16 = 7007;

/// Wasm binary unwrapped. If built with `SKIP_WASM_BUILD`, the function panics.
#[cfg(feature = "std")]
pub fn wasm_binary_unwrap() -> &'static [u8] {
  WASM_BINARY.expect(
    "Development wasm binary is not available. This means the client is \
						built with `SKIP_WASM_BUILD` flag and it is only usable for \
						production chains. Please rebuild with the flag disabled.",
  )
}

/// Runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
  spec_name: create_runtime_str!("tidechain"),
  impl_name: create_runtime_str!("tidechain"),
  authoring_version: 0,
  // This should follow our current node version
  // 1.1 -> 1010
  // 1.10 -> 1100
  // 1.10-1 -> 1101
  // 2.4 -> 2040
  // 2.14 -> 2140
  spec_version: 7000,
  impl_version: 0,
  apis: crate::api::PRUNTIME_API_VERSIONS,
  transaction_version: 1,
  state_version: 0,
};

/// Native version.
#[cfg(any(feature = "std", test))]
pub fn native_version() -> NativeVersion {
  NativeVersion {
    runtime_version: VERSION,
    can_author_with: Default::default(),
  }
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub grandpa: Grandpa,
        pub babe: Babe,
        pub im_online: ImOnline,
        pub authority_discovery: AuthorityDiscovery,
    }
}

pub type CouncilCollectiveInstance = pallet_collective::Instance1;
pub type TechnicalCollectiveInstance = pallet_collective::Instance2;
pub type TechnicalMembershipInstance = pallet_membership::Instance1;

parameter_types! {
  // Pallet Id's
  pub const QuorumPalletId: PalletId = PalletId(*b"py/quorm");
  pub const OraclePalletId: PalletId = PalletId(*b"py/oracl");
  pub const AssetRegistryPalletId: PalletId = PalletId(*b"py/asstr");
  pub const TidefiStakingPalletId: PalletId = PalletId(*b"py/stake");
  pub const FeesPalletId: PalletId = PalletId(*b"py/wfees");
  pub const SunrisePalletId: PalletId = PalletId(*b"py/sunrp");
  pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
  // Sunrise Pool: Number of blocks to wait before they can claim the last era reward.
  // current_era.start_block + Cooldown < current_block to be able to claim last era sunrise reward
  pub const SunriseCooldown: BlockNumber = 1_296_000; // 90 days
}

// FIXME: Should be removed once we'll give control to the community (governance)
impl pallet_sudo::Config for Runtime {
  type RuntimeEvent = RuntimeEvent;
  type RuntimeCall = RuntimeCall;
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = tidefi_primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        // Basic stuff
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,

        // Babe must be before session
        Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned} = 1,

        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 2,
        Indices: pallet_indices::{Pallet, Call, Storage, Config<T>, Event<T>} = 3,
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 4,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 5,

        // Consensus support
        Authorship: pallet_authorship::{Pallet, Storage} = 6,
        Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>} = 7,
        Offences: pallet_offences::{Pallet, Storage, Event} = 8,
        Historical: pallet_session_historical::{Pallet} = 9,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 10,
        Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event, ValidateUnsigned} = 11,
        ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>} = 12,
        AuthorityDiscovery: pallet_authority_discovery::{Pallet, Config} = 13,

        // Governance
        Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>} = 14,
        Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 15,
        TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 16,
        Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>} = 17,
        TechnicalMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 18,
        Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 19,

        // Utility module
        Utility: pallet_utility::{Pallet, Call, Event} = 20,

        // Identity module
        Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 21,

        // Election pallet. Only works with staking, but placed here to maintain indices.
        ElectionProviderMultiPhase: pallet_election_provider_multi_phase::{Pallet, Call, Storage, Event<T>, ValidateUnsigned} = 22,

        // Social recovery module
        Recovery: pallet_recovery::{Pallet, Call, Storage, Event<T>} = 23,

        // System scheduler
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 24,

        // Proxy module
        Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 25,

        // Multisig
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 26,

        // Bounties
        Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>} = 27,

        // Assets
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>} = 28,

        // Provides a semi-sorted list of nominators for staking
        BagsList: pallet_bags_list::{Pallet, Call, Storage, Event<T>} = 29,

        // Preimage registrar
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 30,

        // Sudo module
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 31,

        // Tidefi public calls
        Tidefi: pallet_tidefi::{Pallet, Call, Storage, Event<T>} = 50,

        // Tidefi staking module (assets)
        TidefiStaking: pallet_tidefi_stake::{Pallet, Call, Config<T>, Storage, Event<T>} = 51,

        // Quorum module
        Quorum: pallet_quorum::{Pallet, Call, Config<T>, Storage, Event<T>} = 52,

        // Oracle module
        Oracle: pallet_oracle::{Pallet, Call, Config<T>, Storage, Event<T>} = 53,

        // Security module
        Security: pallet_security::{Pallet, Call, Config, Storage, Event<T>} = 54,

        // Fees module
        Fees: pallet_fees::{Pallet, Config<T>, Storage, Event<T>} = 55,

        // Asset registry module
        AssetRegistry: pallet_asset_registry::{Pallet, Call, Config<T>, Storage, Event<T>} = 56,

        // Sunrise module
        Sunrise: pallet_sunrise::{Pallet, Config<T>, Storage, Event<T>} = 57,

        // Vesting module
        Vesting: pallet_vesting::{Pallet, Storage, Call, Event<T>, Config<T>} = 58,
    }
);

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
  Runtime,
  Block,
  frame_system::ChainContext<Runtime>,
  Runtime,
  AllPalletsWithSystem,
  (
    pallet_bags_list::migrations::AddScore<Runtime>,
    pallet_scheduler::migration::v4::CleanupAgendas<Runtime>,
    // Migrate staking pallet
    MigrateStakingToV8<Runtime>,
    pallet_staking::migrations::v9::InjectValidatorsIntoVoterList<Runtime>,
    pallet_staking::migrations::v10::MigrateToV10<Runtime>,
    pallet_staking::migrations::v11::MigrateToV11<Runtime, BagsList, StakingMigrationV11OldPallet>,
    pallet_staking::migrations::v12::MigrateToV12<Runtime>,
    pallet_staking::migrations::v13::MigrateToV13<Runtime>,
    // Remove stale entries in the set id -> session index storage map (after
    // this release they will be properly pruned after the bonding duration has
    // elapsed)
    pallet_grandpa::migrations::CleanupSetIdSessionMap<Runtime>,
    pallet_election_provider_multi_phase::migrations::v1::MigrateToV1<Runtime>,
    // This migration cleans up empty agendas of the V4 scheduler
    pallet_scheduler::migration::v4::CleanupAgendas<Runtime>,
    // This will drop all calls and refund them (required for the migration)
    pallet_multisig::migrations::v1::MigrateToV1<Runtime>,
    MigrateBountyToV4<Runtime>,
    // Migration for moving preimage from V0 to V1 storage.
    pallet_preimage::migration::v1::Migration<Runtime>,
    // Tidefi migrations
    pallet_tidefi_stake::migrations::v2::MigrateToV2<Runtime>,
    pallet_fees::migrations::v2::MigrateToV2<Runtime>,
  ),
>;

pub struct StakingMigrationV11OldPallet;
impl Get<&'static str> for StakingMigrationV11OldPallet {
  fn get() -> &'static str {
    "BagsList"
  }
}

/// A migration which update `Staking` to `v8`
pub struct MigrateStakingToV8<T>(sp_std::marker::PhantomData<T>);
impl<T: pallet_staking::Config> frame_support::traits::OnRuntimeUpgrade for MigrateStakingToV8<T> {
  fn on_runtime_upgrade() -> Weight {
    pallet_staking::migrations::v8::migrate::<T>()
  }

  #[cfg(feature = "try-runtime")]
  fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
    pallet_staking::migrations::v8::pre_migrate::<T>()?;
    Ok(Vec::new())
  }

  #[cfg(feature = "try-runtime")]
  fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
    pallet_staking::migrations::v8::post_migrate::<T>()
  }
}

/// A migration which update `pallet_bounties` to `v4`
pub struct MigrateBountyToV4<T>(sp_std::marker::PhantomData<T>);
impl<T: pallet_bounties::Config> frame_support::traits::OnRuntimeUpgrade for MigrateBountyToV4<T> {
  fn on_runtime_upgrade() -> Weight {
    pallet_bounties::migrations::v4::migrate::<T, Bounties, &str>("bounties", "bounties")
  }

  #[cfg(feature = "try-runtime")]
  fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
    pallet_bounties::migrations::v4::pre_migration::<T, Bounties, &str>("bounties", "bounties");
    Ok(Vec::new())
  }

  #[cfg(feature = "try-runtime")]
  fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
    pallet_bounties::migrations::v4::post_migration::<T, Bounties, &str>("bounties", "bounties");
    Ok(())
  }
}
