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

use crate::{
  types::{
    AccountId, AuthorityDiscoveryId, Balance, Block, CurrencyId, EpochDuration, GrandpaId, Index,
    BABE_GENESIS_EPOCH_CONFIG,
  },
  AssetRegistry, AuthorityDiscovery, Babe, Executive, Grandpa, Historical, InherentDataExt,
  Runtime, SessionKeys, System, TidefiStaking, TransactionPayment, VERSION,
};
use frame_support::{pallet_prelude::Weight, traits::KeyOwnerProofSystem};
use pallet_grandpa::{fg_primitives, AuthorityList as GrandpaAuthorityList};
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};
use sp_api::{impl_runtime_apis, ApisVec};
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_inherents::{CheckInherentsResult, InherentData};

use sp_runtime::{
  traits::{Block as BlockT, NumberFor},
  transaction_validity::{TransactionSource, TransactionValidity},
  ApplyExtrinsicResult, DispatchError,
};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;
use tidefi_primitives::{BalanceInfo, BlockNumber, CurrencyBalance, CurrencyMetadata, Stake};

// Work around the issue that RUNTIME_API_VERSIONS is not public.
pub(crate) const PRUNTIME_API_VERSIONS: ApisVec = RUNTIME_API_VERSIONS;

impl_runtime_apis! {
   impl sp_api::Core<Block> for Runtime {
       fn version() -> RuntimeVersion {
           VERSION
       }

       fn execute_block(block: Block) {
           Executive::execute_block(block);
       }

       fn initialize_block(header: &<Block as BlockT>::Header) {
           Executive::initialize_block(header)
       }
   }

   impl sp_api::Metadata<Block> for Runtime {
       fn metadata() -> OpaqueMetadata {
           OpaqueMetadata::new(Runtime::metadata().into())
       }
   }

   impl sp_block_builder::BlockBuilder<Block> for Runtime {
       fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
           Executive::apply_extrinsic(extrinsic)
       }

       fn finalize_block() -> <Block as BlockT>::Header {
           Executive::finalize_block()
       }

       fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
           data.create_extrinsics()
       }

       fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult {
           data.check_extrinsics(&block)
       }
   }

   impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
       fn validate_transaction(
           source: TransactionSource,
           tx: <Block as BlockT>::Extrinsic,
           block_hash: <Block as BlockT>::Hash,
       ) -> TransactionValidity {
           Executive::validate_transaction(source, tx, block_hash)
       }
   }

   impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
       fn offchain_worker(header: &<Block as BlockT>::Header) {
           Executive::offchain_worker(header)
       }
   }

   impl fg_primitives::GrandpaApi<Block> for Runtime {
       fn grandpa_authorities() -> GrandpaAuthorityList {
           Grandpa::grandpa_authorities()
       }

       fn current_set_id() -> fg_primitives::SetId {
         Grandpa::current_set_id()
       }

       fn submit_report_equivocation_unsigned_extrinsic(
           equivocation_proof: fg_primitives::EquivocationProof<
               <Block as BlockT>::Hash,
               NumberFor<Block>,
           >,
           key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
       ) -> Option<()> {
           let key_owner_proof = key_owner_proof.decode()?;

           Grandpa::submit_unsigned_equivocation_report(
               equivocation_proof,
               key_owner_proof,
           )
       }

       fn generate_key_ownership_proof(
           _set_id: fg_primitives::SetId,
           authority_id: GrandpaId,
       ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
           use codec::Encode;

           Historical::prove((fg_primitives::KEY_TYPE, authority_id))
               .map(|p| p.encode())
               .map(fg_primitives::OpaqueKeyOwnershipProof::new)
       }
   }

   impl sp_consensus_babe::BabeApi<Block> for Runtime {
       fn configuration() -> sp_consensus_babe::BabeConfiguration {
           // The choice of `c` parameter (where `1 - c` represents the
           // probability of a slot being empty), is done in accordance to the
           // slot duration and expected target block time, for safely
           // resisting network delays of maximum two seconds.
           // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
           sp_consensus_babe::BabeConfiguration {
               slot_duration: Babe::slot_duration(),
               epoch_length: EpochDuration::get(),
               c: BABE_GENESIS_EPOCH_CONFIG.c,
               authorities: Babe::authorities().to_vec(),
               randomness: Babe::randomness(),
               allowed_slots: BABE_GENESIS_EPOCH_CONFIG.allowed_slots,
           }
       }

       fn current_epoch_start() -> sp_consensus_babe::Slot {
           Babe::current_epoch_start()
       }

       fn current_epoch() -> sp_consensus_babe::Epoch {
           Babe::current_epoch()
       }

       fn next_epoch() -> sp_consensus_babe::Epoch {
           Babe::next_epoch()
       }

       fn generate_key_ownership_proof(
           _slot: sp_consensus_babe::Slot,
           authority_id: sp_consensus_babe::AuthorityId,
       ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
           use codec::Encode;

           Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
               .map(|p| p.encode())
               .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
       }

       fn submit_report_equivocation_unsigned_extrinsic(
           equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
           key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
       ) -> Option<()> {
           let key_owner_proof = key_owner_proof.decode()?;

           Babe::submit_unsigned_equivocation_report(
               equivocation_proof,
               key_owner_proof,
           )
       }
   }

   impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
       fn authorities() -> Vec<AuthorityDiscoveryId> {
           AuthorityDiscovery::authorities()
       }
   }

   impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
       fn account_nonce(account: AccountId) -> Index {
           System::account_nonce(account)
       }
   }

   impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
       Block,
       Balance,
   > for Runtime {
       fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
           TransactionPayment::query_info(uxt, len)
       }
       fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
           TransactionPayment::query_fee_details(uxt, len)
       }
       fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
       }
       fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
       }
   }

   impl sp_session::SessionKeys<Block> for Runtime {
       fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
           SessionKeys::generate(seed)
       }

       fn decode_session_keys(
           encoded: Vec<u8>,
       ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
           SessionKeys::decode_into_raw_public_keys(&encoded)
       }
   }

   // Tidefi Custom API
   impl pallet_tidefi_rpc_runtime_api::TidefiApi<Block, AccountId> for Runtime {
     fn get_assets() -> Result<Vec<(CurrencyId, CurrencyMetadata<Vec<u8>>)>, DispatchError> {
       AssetRegistry::get_assets()
     }
     fn get_account_balance(account_id: AccountId, asset_id: CurrencyId) -> Result<CurrencyBalance<BalanceInfo>, DispatchError> {
       AssetRegistry::get_account_balance(&account_id, asset_id)
     }
     fn get_account_balances(account_id: AccountId) -> Result<Vec<(CurrencyId, CurrencyBalance<BalanceInfo>)>, DispatchError> {
       AssetRegistry::get_account_balances(&account_id)
     }
     fn get_account_stakes(account_id: AccountId) -> Result<Vec<(CurrencyId, Stake<BalanceInfo, BlockNumber>)>, DispatchError> {
       Ok(TidefiStaking::get_account_stakes(&account_id))
     }
   }

   #[cfg(feature = "try-runtime")]
   impl frame_try_runtime::TryRuntime<Block> for Runtime {
     fn on_runtime_upgrade() -> (frame_support::weights::Weight, frame_support::weights::Weight) {
        log::info!("try-runtime::on_runtime_upgrade tidechain.");
        let weight = Executive::try_runtime_upgrade().unwrap();
        (weight, crate::types::RuntimeBlockWeights::get().max_block)
     }
     fn execute_block_no_check(block: Block) -> frame_support::weights::Weight {
        Executive::execute_block_no_check(block)
     }
   }

   #[cfg(feature = "runtime-benchmarks")]
   impl frame_benchmarking::Benchmark<Block> for Runtime {
     fn benchmark_metadata(extra: bool) -> (
       Vec<frame_benchmarking::BenchmarkList>,
       Vec<frame_support::traits::StorageInfo>,
     ) {
       use frame_benchmarking::v1::{list_benchmark, Benchmarking, BenchmarkList};
       use frame_support::traits::StorageInfoTrait;

       use pallet_session_benchmarking::Pallet as SessionBench;
       //use pallet_offences_benchmarking::Pallet as OffencesBench;
       use frame_system_benchmarking::Pallet as SystemBench;
       use pallet_election_provider_support_benchmarking::Pallet as ElectionProviderBench;

       let mut list = Vec::<BenchmarkList>::new();

       //list_benchmark!(list, extra, pallet_bags_list, crate::BagsList);
       list_benchmark!(list, extra, pallet_balances, crate::Balances);
       list_benchmark!(list, extra, pallet_bounties, crate::Bounties);
       list_benchmark!(list, extra, pallet_democracy, crate::Democracy);
       list_benchmark!(list, extra, pallet_collective, crate::Council);
       list_benchmark!(list, extra, pallet_election_provider_multi_phase, crate::ElectionProviderMultiPhase);
       list_benchmark!(list, extra, pallet_elections_phragmen, crate::Elections);
       //list_benchmark!(list, extra, pallet_grandpa, Grandpa);
       list_benchmark!(list, extra, pallet_identity, crate::Identity);
       list_benchmark!(list, extra, pallet_im_online, crate::ImOnline);
       list_benchmark!(list, extra, pallet_indices, crate::Indices);
       list_benchmark!(list, extra, pallet_membership, crate::TechnicalMembership);
       list_benchmark!(list, extra, pallet_multisig, crate::Multisig);
       //list_benchmark!(list, extra, pallet_offences, OffencesBench::<Runtime>);
       //list_benchmark!(list, extra, pallet_proxy, crate::Proxy);
       list_benchmark!(list, extra, pallet_preimage, crate::Preimage);
       list_benchmark!(list, extra, pallet_scheduler, crate::Scheduler);
       list_benchmark!(list, extra, pallet_session, SessionBench::<Runtime>);
       list_benchmark!(list, extra, pallet_security, crate::Security);
       list_benchmark!(list, extra, pallet_staking, crate::Staking);
       list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
       list_benchmark!(list, extra, pallet_assets, crate::Assets);
       list_benchmark!(list, extra, pallet_timestamp, crate::Timestamp);
       list_benchmark!(list, extra, pallet_treasury, crate::Treasury);
       list_benchmark!(list, extra, pallet_utility, crate::Utility);
       list_benchmark!(list, extra, pallet_tidefi, crate::Tidefi);
       list_benchmark!(list, extra, pallet_quorum, crate::Quorum);
       list_benchmark!(list, extra, pallet_tidefi_stake, crate::TidefiStaking);
       list_benchmark!(list, extra, pallet_oracle, crate::Oracle);
       list_benchmark!(list, extra, pallet_asset_registry, crate::AssetRegistry);
       list_benchmark!(list, extra, frame_election_provider_support, ElectionProviderBench::<Runtime>);
       list_benchmark!(list, extra, pallet_fees, crate::Fees);

       let storage_info = crate::AllPalletsWithSystem::storage_info();

       (list, storage_info)
     }

       fn dispatch_benchmark(
           config: frame_benchmarking::BenchmarkConfig
       ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
           use frame_benchmarking::v1::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};
           // Trying to add benchmarks directly to the Session Pallet caused cyclic dependency
           // issues. To get around that, we separated the Session benchmarks into its own crate,
           // which is why we need these two lines below.
           use pallet_session_benchmarking::Pallet as SessionBench;
           impl pallet_session_benchmarking::Config for Runtime {}

           use frame_system_benchmarking::Pallet as SystemBench;
           impl frame_system_benchmarking::Config for Runtime {}
           use pallet_election_provider_support_benchmarking::Pallet as ElectionProviderBench;
           impl pallet_election_provider_support_benchmarking::Config for Runtime {}

           //use pallet_offences_benchmarking::Pallet as OffencesBench;
           //impl pallet_offences_benchmarking::Config for Runtime {}


           let whitelist: Vec<TrackedStorageKey> = vec![
               // Block Number
               hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
               // Total Issuance
               hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
               // Execution Phase
               hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
               // Event Count
               hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
               // System Events
               hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
               // Treasury Account
               hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").to_vec().into(),
           ];

           let mut batches = Vec::<BenchmarkBatch>::new();
           let params = (&config, &whitelist);

           add_benchmark!(params, batches, pallet_assets, crate::Assets);
           // FIXME: BagsList benchs are failing (use default provided by substrate)
           //add_benchmark!(params, batches, pallet_bags_list, crate::BagsList);
           add_benchmark!(params, batches, pallet_balances, crate::Balances);
           add_benchmark!(params, batches, pallet_bounties, crate::Bounties);
           add_benchmark!(params, batches, pallet_democracy, crate::Democracy);
           add_benchmark!(params, batches, pallet_collective, crate::Council);
           add_benchmark!(params, batches, pallet_election_provider_multi_phase, crate::ElectionProviderMultiPhase);
           add_benchmark!(params, batches, pallet_elections_phragmen, crate::Elections);
           // FIXME: pallet_grandpa benchmark do not generate the correct functions
           //add_benchmark!(params, batches, pallet_grandpa, Grandpa);
           add_benchmark!(params, batches, pallet_identity, crate::Identity);
           add_benchmark!(params, batches, pallet_im_online, crate::ImOnline);
           add_benchmark!(params, batches, pallet_indices, crate::Indices);
           add_benchmark!(params, batches, pallet_membership, crate::TechnicalMembership);
           add_benchmark!(params, batches, pallet_multisig, crate::Multisig);
           //add_benchmark!(params, batches, pallet_offences, OffencesBench::<Runtime>);
           //add_benchmark!(params, batches, pallet_proxy, crate::Proxy);
           add_benchmark!(params, batches, pallet_scheduler, crate::Scheduler);
           add_benchmark!(params, batches, pallet_session, SessionBench::<Runtime>);
           add_benchmark!(params, batches, pallet_staking, crate::Staking);
           add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
           add_benchmark!(params, batches, pallet_security, crate::Security);
           add_benchmark!(params, batches, pallet_timestamp, crate::Timestamp);
           add_benchmark!(params, batches, pallet_treasury, crate::Treasury);
           add_benchmark!(params, batches, pallet_utility, crate::Utility);
           add_benchmark!(params, batches, pallet_tidefi, crate::Tidefi);
           add_benchmark!(params, batches, pallet_tidefi_stake, crate::TidefiStaking);
           add_benchmark!(params, batches, pallet_quorum, crate::Quorum);
           add_benchmark!(params, batches, pallet_oracle, crate::Oracle);
           add_benchmark!(params, batches, pallet_asset_registry, crate::AssetRegistry);
           add_benchmark!(params, batches, pallet_preimage, crate::Preimage);
           add_benchmark!(params, batches, frame_election_provider_support, ElectionProviderBench::<Runtime>);
           add_benchmark!(params, batches, pallet_fees, crate::Fees);

           if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
           Ok(batches)
       }
   }
}
