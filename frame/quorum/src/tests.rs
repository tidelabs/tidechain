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
  mock::{
    new_test_ext, Origin, ProposalLifetime, PubkeyLimitPerAsset, Quorum, Security, StringLimit,
    System, Test,
  },
  pallet::*,
};
use frame_support::{assert_noop, assert_ok, traits::Hooks, BoundedVec};
use pallet_security::CurrentBlockCount as CurrentBlockNumber;
use tidefi_primitives::{
  pallet::SecurityExt, ComplianceLevel, CurrencyId, Hash, Mint, ProposalType,
};

struct Context {
  alice: Origin,
  public_keys: BoundedVec<(u64, BoundedVec<u8, StringLimit>), PubkeyLimitPerAsset>,
  proposal_id: Hash,
}

impl Default for Context {
  fn default() -> Self {
    let pub_key: BoundedVec<u8, StringLimit> = "pubkey".as_bytes().to_vec().try_into().unwrap();
    Self {
      alice: Origin::signed(1u64),
      public_keys: vec![(1u64, pub_key)].try_into().unwrap(),
      proposal_id: Hash::zero(),
    }
  }
}

impl Context {
  fn setup(&self) {
    PublicKeys::<Test>::insert(1, self.public_keys.clone());
    assert!(Members::<Test>::contains_key(1));
    assert_eq!(PublicKeys::<Test>::get(1).len(), 1);
  }
}

fn set_current_block(block_number: u64) {
  <CurrentBlockNumber<Test>>::mutate(|n| {
    *n = block_number;
    *n
  });
}

#[test]
pub fn should_submit_proposal() {
  new_test_ext().execute_with(|| {
    let context = Context::default();
    context.setup();

    let proposal = ProposalType::Mint(Mint {
      account_id: 1,
      currency_id: CurrencyId::Tdfy,
      mint_amount: 1_000_000_000_000,
      transaction_id: Vec::new(),
      compliance_level: ComplianceLevel::Green,
    });
    assert_ok!(Quorum::submit_proposal(context.alice, proposal));
  });
}

#[test]
pub fn should_vote_for_mint() {
  new_test_ext().execute_with(|| {
    let context = Context::default();
    context.setup();

    let proposal = ProposalType::Mint(Mint {
      account_id: 1,
      currency_id: CurrencyId::Tdfy,
      mint_amount: 1_000_000_000_000,
      transaction_id: Default::default(),
      compliance_level: ComplianceLevel::Green,
    });

    assert_ok!(Proposals::<Test>::try_append((
      context.proposal_id,
      Security::get_current_block_count(),
      proposal
    )));
    assert_ok!(Quorum::acknowledge_proposal(
      context.alice,
      context.proposal_id
    ));
  });
}

#[test]
pub fn should_remove_expired() {
  new_test_ext().execute_with(|| {
    let context = Context::default();
    context.setup();

    let proposal = ProposalType::Mint(Mint {
      account_id: 1,
      currency_id: CurrencyId::Tdfy,
      mint_amount: 1_000_000_000_000,
      transaction_id: Default::default(),
      compliance_level: ComplianceLevel::Green,
    });
    assert_ok!(Quorum::submit_proposal(context.alice, proposal));
    assert_eq!(Quorum::on_idle(0, 1_000_000_000_000), 0);
    assert_eq!(Proposals::<Test>::get().len(), 1);

    set_current_block(ProposalLifetime::get() + 2);

    assert_eq!(Quorum::on_idle(0, 1_000_000_000_000), 0);
    assert_eq!(Proposals::<Test>::get().len(), 0);
  });
}

#[test]
pub fn test_vec_shuffle() {
  // switching block hash should give new shuffle
  new_test_ext().execute_with(|| {
    let block_hash = Security::get_unique_id(1_u64);
    System::set_parent_hash(block_hash);
    assert_eq!(Quorum::create_shuffle(4), vec![3, 2, 1, 0]);

    let block_hash = Security::get_unique_id(2_u64);
    System::set_parent_hash(block_hash);
    assert_eq!(Quorum::create_shuffle(4), vec![2, 0, 3, 1]);

    let block_hash = Security::get_unique_id(3_u64);
    System::set_parent_hash(block_hash);
    assert_eq!(Quorum::create_shuffle(4), vec![3, 2, 0, 1]);
  });
}

mod vote_should_fail_for {
  use super::*;

  #[test]
  pub fn non_existent_proposal() {
    new_test_ext().execute_with(|| {
      let context = Context::default();
      context.setup();

      assert_noop!(
        Quorum::acknowledge_proposal(context.alice.clone(), context.proposal_id),
        Error::<Test>::ProposalDoesNotExist
      );
      assert_noop!(
        Quorum::reject_proposal(context.alice, context.proposal_id),
        Error::<Test>::ProposalDoesNotExist
      );
    });
  }

  #[test]
  pub fn future_proposal() {
    new_test_ext().execute_with(|| {
      let context = Context::default();
      context.setup();

      let proposal = ProposalType::Mint(Mint {
        account_id: 1,
        currency_id: CurrencyId::Tdfy,
        mint_amount: 1_000_000_000_000,
        transaction_id: Default::default(),
        compliance_level: ComplianceLevel::Green,
      });

      assert_ok!(Proposals::<Test>::try_append((
        context.proposal_id,
        Security::get_current_block_count() + 100,
        proposal
      )));

      assert_noop!(
        Quorum::acknowledge_proposal(context.alice.clone(), context.proposal_id),
        Error::<Test>::ProposalBlockIsInFuture
      );
      assert_noop!(
        Quorum::reject_proposal(context.alice, context.proposal_id),
        Error::<Test>::ProposalBlockIsInFuture
      );
    });
  }

  #[test]
  pub fn expired_proposal() {
    new_test_ext().execute_with(|| {
      let context = Context::default();
      context.setup();

      let proposal = ProposalType::Mint(Mint {
        account_id: 1,
        currency_id: CurrencyId::Tdfy,
        mint_amount: 1_000_000_000_000,
        transaction_id: Default::default(),
        compliance_level: ComplianceLevel::Green,
      });

      let current_block = Security::get_current_block_count();
      assert_ok!(Proposals::<Test>::try_append((
        context.proposal_id,
        current_block,
        proposal
      )));

      set_current_block(current_block + ProposalLifetime::get() + 1);

      assert_noop!(
        Quorum::acknowledge_proposal(context.alice.clone(), context.proposal_id),
        Error::<Test>::ProposalExpired
      );
      assert_noop!(
        Quorum::reject_proposal(context.alice, context.proposal_id),
        Error::<Test>::ProposalExpired
      );
    });
  }
}
