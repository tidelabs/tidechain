#![allow(clippy::unnecessary_cast)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::unused_unit)]
#![allow(clippy::double_parens)]

//! A list of the different weight modules for our runtime.

pub mod frame_system;
pub mod pallet_asset_registry;
pub mod pallet_assets;
pub mod pallet_bags_list;
pub mod pallet_balances;
pub mod pallet_bounties;
pub mod pallet_collective;
pub mod pallet_election_provider_multi_phase;
pub mod pallet_elections_phragmen;
pub mod pallet_grandpa;
pub mod pallet_identity;
pub mod pallet_im_online;
pub mod pallet_indices;
pub mod pallet_membership;
pub mod pallet_multisig;
pub mod pallet_oracle;
pub mod pallet_preimage;
pub mod pallet_proxy;
pub mod pallet_quorum;
pub mod pallet_scheduler;
pub mod pallet_session;
pub mod pallet_staking;
pub mod pallet_tidefi;
pub mod pallet_tidefi_stake;
pub mod pallet_timestamp;
pub mod pallet_treasury;
pub mod pallet_utility;
