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
  // This should follow our current node version
  // 1.1 -> 1010
  // 1.10 -> 1100
  // 1.10-1 -> 1101
  // 2.4 -> 2040
  // 2.14 -> 2140
  spec_version: 1060,
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
  pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
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
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 5,

        // Consensus support
        Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent} = 6,
        Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>} = 7,
        Offences: pallet_offences::{Pallet, Storage, Event} = 8,
        Historical: pallet_session_historical::{Pallet} = 9,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 10,
        Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event, ValidateUnsigned} = 11,
        ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>} = 12,
        AuthorityDiscovery: pallet_authority_discovery::{Pallet, Config} = 13,

        // Governance
        Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 14,
        TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 15,
        Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>} = 16,
        TechnicalMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 17,
        Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 18,

        // Utility module
        Utility: pallet_utility::{Pallet, Call, Event} = 19,

        // Identity module
        Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 20,

        // Election pallet. Only works with staking, but placed here to maintain indices.
        ElectionProviderMultiPhase: pallet_election_provider_multi_phase::{Pallet, Call, Storage, Event<T>, ValidateUnsigned} = 21,

        // Social recovery module
        Recovery: pallet_recovery::{Pallet, Call, Storage, Event<T>} = 22,

        // System scheduler
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 23,

        // Proxy module
        Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 24,

        // Multisig
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 25,

        // Bounties
        Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>} = 26,

        // Assets
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>} = 27,

        // Provides a semi-sorted list of nominators for staking
        BagsList: pallet_bags_list::{Pallet, Call, Storage, Event<T>} = 28,

        // Preimage registrar
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 29,

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
        Fees: pallet_fees::{Pallet, Call, Config<T>, Storage, Event<T>} = 55,

        // Asset registry module
        AssetRegistry: pallet_asset_registry::{Pallet, Call, Config<T>, Storage, Event<T>} = 56,
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
