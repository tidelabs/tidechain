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
  BlockLength, BlockWeights, Call, CouncilCollectiveInstance, Runtime, RuntimeVersion, VERSION,
};
use frame_support::{
  parameter_types,
  traits::{Currency, EitherOfDiverse},
  weights::{constants::WEIGHT_PER_SECOND, DispatchClass, Weight},
};
use frame_system::EnsureRoot;
use sp_runtime::{generic, Perbill};
use static_assertions::const_assert;

pub use frame_support::weights::constants::{
  BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight,
};
pub use pallet_grandpa::AuthorityId as GrandpaId;
pub use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
pub use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
pub use sp_runtime::traits::BlakeTwo256;

pub use tidefi_primitives::{
  AccountId, AccountIndex, AssetId, Balance, BlockNumber, CurrencyId, Hash, Index, Moment,
  SessionIndex, Signature,
};

pub type NegativeImbalance<T> = <pallet_balances::Pallet<T> as Currency<
  <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

/// The BABE epoch configuration at genesis.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
  sp_consensus_babe::BabeEpochConfiguration {
    c: crate::constants::time::PRIMARY_PROBABILITY,
    allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryVRFSlots,
  };

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 4 seconds of compute with a 12 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = 4 * WEIGHT_PER_SECOND;

const_assert!(NORMAL_DISPATCH_RATIO.deconstruct() >= AVERAGE_ON_INITIALIZE_RATIO.deconstruct());

parameter_types! {
   pub const BlockHashCount: BlockNumber = 2400;
   pub const Version: RuntimeVersion = VERSION;
   pub const EpochDuration: u64 = crate::constants::time::EPOCH_DURATION_IN_SLOTS as u64;
   pub RuntimeBlockLength: BlockLength =
       BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
   pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
       .base_block(BlockExecutionWeight::get())
       .for_class(DispatchClass::all(), |weights| {
           weights.base_extrinsic = ExtrinsicBaseWeight::get();
       })
       .for_class(DispatchClass::Normal, |weights| {
           weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
       })
       .for_class(DispatchClass::Operational, |weights| {
           weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
           // Operational transactions have some extra reserved space, so that they
           // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
           weights.reserved = Some(
               MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
           );
       })
       .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
       .build_or_panic();
}

pub type EnsureRootOrHalfCouncil = EitherOfDiverse<
  EnsureRoot<AccountId>,
  pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollectiveInstance, 1, 2>,
>;

pub type CurrencyToVote = frame_support::traits::U128CurrencyToVote;
pub type Nonce = u32;

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, AccountIndex>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
  frame_system::CheckSpecVersion<Runtime>,
  frame_system::CheckTxVersion<Runtime>,
  frame_system::CheckGenesis<Runtime>,
  frame_system::CheckMortality<Runtime>,
  frame_system::CheckNonce<Runtime>,
  frame_system::CheckWeight<Runtime>,
  pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
