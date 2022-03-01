//! Benchmarking setup for pallet-tidefi

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::{
  account, benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller, Vec,
};
use frame_system::{self, RawOrigin};
use tidefi_primitives::{
  pallet::SecurityExt, ComplianceLevel, CurrencyId, Hash, Mint, ProposalType, ProposalVotes,
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
  PublicKeys::<T>::insert(1, vec![(user.clone(), "pubkey".as_bytes().to_vec())]);
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
   eval_proposal_state {
      let user = pre_set_auth::<T>();
      let proposal_id = create_proposal::<T>();
      Votes::<T>::insert(proposal_id, ProposalVotes::default());
   }: _(RawOrigin::Signed(user), proposal_id)
   submit_public_keys {
      // test with 100 assets
      let k in 0 .. 100;
      let mut all_keys = vec![];
      for i in 0..k {
         all_keys.push((i as u32, "new_pubkey".as_bytes().to_vec()));
      }
      let user = pre_set_auth::<T>();
   }: _(RawOrigin::Signed(user), all_keys)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
