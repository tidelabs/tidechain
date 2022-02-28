//! Benchmarking setup for pallet-tidefi

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller, Vec};
use frame_system::{self, RawOrigin};
use tidefi_primitives::{
  pallet::SecurityExt, ComplianceLevel, CurrencyId, Hash, Mint, ProposalType,
};

const SEED: u32 = 0;
const ADMIN_ID: u32 = 1;

fn _assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
  frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn pre_set_auth<T: Config>() -> T::AccountId {
  let user: T::AccountId = account("admin", ADMIN_ID, SEED);
  Members::<T>::remove_all();
  Members::<T>::insert(&user, true);
  PublicKeys::<T>::insert(&user, 1, "pubkey".as_bytes());
  Threshold::<T>::put(1);
  user
}

fn create_proposal<T: Config>() -> Hash {
  let account_id: T::AccountId = whitelisted_caller();
  let proposal = ProposalType::Mint(Mint {
    account_id,
    currency_id: CurrencyId::Tide,
    mint_amount: 1_000_000_000_000,
    transaction_id: Vec::new(),
    compliance_level: ComplianceLevel::Green,
  });

  let proposal_id = Hash::zero();
  Proposals::<T>::try_append((
    proposal_id,
    T::Security::get_current_block_count(),
    proposal,
  ))
  .unwrap();

  proposal_id
}

benchmarks! {
   submit_proposal {
      let user = pre_set_auth::<T>();
      let account_id: T::AccountId = whitelisted_caller();
      let proposal = ProposalType::Mint(Mint {
         account_id,
         currency_id: CurrencyId::Tide,
         mint_amount: 1_000_000_000_000,
         transaction_id: Vec::new(),
         compliance_level: ComplianceLevel::Green,
      });
   }: _(RawOrigin::Signed(user), proposal)
   acknowledge_proposal {
      let user = pre_set_auth::<T>();
      let proposal_id = create_proposal::<T>();
   }: _(RawOrigin::Signed(user), proposal_id)
   reject_proposal {
      let user = pre_set_auth::<T>();
      let proposal_id = create_proposal::<T>();
   }: _(RawOrigin::Signed(user), proposal_id)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
