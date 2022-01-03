#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
#![allow(clippy::from_over_into)]
#![allow(clippy::manual_range_contains)]

use frame_support::{construct_runtime, parameter_types};

use frame_support::PalletId;
use frame_system::limits::{BlockLength, BlockWeights};

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
#[path = "../../common/bag_thresholds.rs"]
mod bag_thresholds;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// FIXME: Replace with SEMNET Unique prefix. (7007)
// 42 = Substrate default
pub const SS58_PREFIX: u8 = 42;

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
  // Per convention: if the runtime behavior changes, increment spec_version
  // and set impl_version to 0. If only runtime
  // implementation changes and behavior does not, then leave spec_version as
  // is and increment impl_version.
  spec_version: 1,
  impl_version: 0,
  apis: crate::api::PRUNTIME_API_VERSIONS,
  transaction_version: 1,
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
  pub const WraprStakePalletId: PalletId = PalletId(*b"py/stake");
  pub const FeesPalletId: PalletId = PalletId(*b"py/wfees");
  pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = tidefi_primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
        Utility: pallet_utility::{Pallet, Call, Event} = 1,
        Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned} = 2,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 3,
        Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent} = 4,
        Indices: pallet_indices::{Pallet, Call, Storage, Config<T>, Event<T>} = 5,
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 6,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 7,
        ElectionProviderMultiPhase: pallet_election_provider_multi_phase::{Pallet, Call, Storage, Event<T>, ValidateUnsigned} = 8,
        Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>} = 9,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 10,
        Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 11,
        TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 12,
        Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>} = 13,
        TechnicalMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 14,
        Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event, ValidateUnsigned} = 15,
        Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 16,
        ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>} = 18,
        AuthorityDiscovery: pallet_authority_discovery::{Pallet, Config} = 19,
        Offences: pallet_offences::{Pallet, Storage, Event} = 20,
        Historical: pallet_session_historical::{Pallet} = 21,
        Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 23,
        Recovery: pallet_recovery::{Pallet, Call, Storage, Event<T>} = 24,
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 25,
        Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 26,
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 27,
        Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>} = 28,
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>} = 29,
        // Request handler for withdrawls
        Wrapr: pallet_wrapr::{Pallet, Call, Storage, Event<T>} = 30,
        // Staking storage and hooks
        WraprStake: pallet_wrapr_stake::{Pallet, Call, Storage, Event<T>} = 31,
        // Storage, events and traits for the quorum
        WraprQuorum: pallet_quorum::{Pallet, Call, Config<T>, Storage, Event<T>} = 32,
        // Storage, events and traits for the oracle
        WraprOracle: pallet_oracle::{Pallet, Call, Config<T>, Storage, Event<T>} = 33,
        // Storage, events and traits for the securities utils
        WraprSecurity: pallet_security::{Pallet, Call, Config, Storage, Event<T>} = 34,
        // Storage, events and traits for the asset registry
        WraprAssetRegistry: pallet_asset_registry::{Pallet, Call, Config<T>, Storage, Event<T>} = 35,
        // Storage, events and traits for the fees
        WraprFees: pallet_fees::{Pallet, Config<T>, Storage, Event<T>} = 36,
        // Provides a semi-sorted list of nominators for staking.
        BagsList: pallet_bags_list::{Pallet, Call, Storage, Event<T>} = 37,
        // Preimage registrar.
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 38,
    }
);

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
  Runtime,
  Block,
  frame_system::ChainContext<Runtime>,
  Runtime,
  AllPalletsWithSystem,
  (),
>;
