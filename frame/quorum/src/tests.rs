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
use sp_runtime::traits::BadOrigin;
use tidefi_primitives::{
  pallet::SecurityExt, AssetId, ComplianceLevel, CurrencyId, Hash, Mint, ProposalType,
};

const ASSET_1: AssetId = 1u32;
const ASSET_2: AssetId = 2u32;
const ALICE_ACCOUNT_ID: u32 = 1;
const BOB_ACCOUNT_ID: u32 = 2;
const ONE_TDFY: u128 = 1_000_000_000_000;

type AccountId = u64;

struct Context {
  alice: Origin,
  bob: Origin,
  pub_key: Vec<u8>,
  public_keys: BoundedVec<(AccountId, BoundedVec<u8, StringLimit>), PubkeyLimitPerAsset>,
  proposal_id: Hash,
}

impl Default for Context {
  fn default() -> Self {
    let pub_key_bounded_vec: BoundedVec<u8, StringLimit> =
      "pubkey".as_bytes().to_vec().try_into().unwrap();
    Self {
      alice: Origin::signed(ALICE_ACCOUNT_ID.into()),
      bob: Origin::signed(BOB_ACCOUNT_ID.into()),
      pub_key: pub_key_bounded_vec.to_vec(),
      public_keys: vec![(ALICE_ACCOUNT_ID.into(), pub_key_bounded_vec)]
        .try_into()
        .unwrap(),
      proposal_id: Hash::zero(),
    }
  }
}

impl Context {
  fn insert_asset1_with_alice_public_key(self) -> Self {
    PublicKeys::<Test>::insert(ASSET_1, self.public_keys.clone());
    assert!(Members::<Test>::contains_key(ALICE_ACCOUNT_ID as u64));
    assert_eq!(PublicKeys::<Test>::get(ASSET_1).len(), 1);
    self
  }
}

fn set_current_block(block_number: u64) {
  <CurrentBlockNumber<Test>>::mutate(|n| {
    *n = block_number;
    *n
  });
}

mod submit_proposal {
  use super::*;

  #[test]
  pub fn succeeds() {
    new_test_ext().execute_with(|| {
      let context = Context::default().insert_asset1_with_alice_public_key();

      let proposal = ProposalType::Mint(Mint {
        account_id: ALICE_ACCOUNT_ID.into(),
        currency_id: CurrencyId::Tdfy,
        mint_amount: ONE_TDFY,
        transaction_id: Vec::new(),
        compliance_level: ComplianceLevel::Green,
      });
      assert_ok!(Quorum::submit_proposal(context.alice, proposal));
    });
  }
}

#[test]
pub fn should_vote_for_mint() {
  new_test_ext().execute_with(|| {
    let context = Context::default().insert_asset1_with_alice_public_key();

    let proposal = ProposalType::Mint(Mint {
      account_id: ALICE_ACCOUNT_ID.into(),
      currency_id: CurrencyId::Tdfy,
      mint_amount: ONE_TDFY,
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
    let context = Context::default().insert_asset1_with_alice_public_key();

    let proposal = ProposalType::Mint(Mint {
      account_id: ALICE_ACCOUNT_ID.into(),
      currency_id: CurrencyId::Tdfy,
      mint_amount: ONE_TDFY,
      transaction_id: Default::default(),
      compliance_level: ComplianceLevel::Green,
    });
    assert_ok!(Quorum::submit_proposal(context.alice, proposal));
    assert_eq!(Quorum::on_idle(0, ONE_TDFY.try_into().unwrap()), 0);
    assert_eq!(Proposals::<Test>::get().len(), 1);

    set_current_block(ProposalLifetime::get() + 2);

    assert_eq!(Quorum::on_idle(0, ONE_TDFY.try_into().unwrap()), 0);
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

mod submit_public_keys {
  use super::*;

  #[test]
  pub fn succeeds() {
    new_test_ext().execute_with(|| {
      let context = Context::default().insert_asset1_with_alice_public_key();

      assert_ok!(Quorum::submit_public_keys(
        context.alice,
        vec![(ASSET_2, context.pub_key.clone())]
      ));

      assert!(Quorum::public_keys(ASSET_1).is_empty());

      let asset2_pub_key = Quorum::public_keys(ASSET_2).get(0).unwrap().clone();
      assert_eq!(ALICE_ACCOUNT_ID as u64, asset2_pub_key.0);
      assert_eq!(context.pub_key, asset2_pub_key.1.into_inner());
    });
  }

  mod fails_when {
    use super::*;

    #[test]
    pub fn sender_is_not_signed() {
      new_test_ext().execute_with(|| {
        let context = Context::default().insert_asset1_with_alice_public_key();

        assert_noop!(
          Quorum::submit_public_keys(Origin::none(), vec![(ASSET_2, context.pub_key)]),
          BadOrigin
        );
      });
    }

    #[test]
    pub fn sender_is_not_a_member() {
      new_test_ext().execute_with(|| {
        let context = Context::default().insert_asset1_with_alice_public_key();

        assert_noop!(
          Quorum::submit_public_keys(context.bob, vec![(ASSET_2, context.pub_key)]),
          Error::<Test>::AccessDenied
        );
      });
    }
  }
}

mod vote_should_fail_for {
  use super::*;

  #[test]
  pub fn non_existent_proposal() {
    new_test_ext().execute_with(|| {
      let context = Context::default().insert_asset1_with_alice_public_key();

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
      let context = Context::default().insert_asset1_with_alice_public_key();

      let proposal = ProposalType::Mint(Mint {
        account_id: ALICE_ACCOUNT_ID.into(),
        currency_id: CurrencyId::Tdfy,
        mint_amount: ONE_TDFY,
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
      let context = Context::default().insert_asset1_with_alice_public_key();

      let proposal = ProposalType::Mint(Mint {
        account_id: ALICE_ACCOUNT_ID.into(),
        currency_id: CurrencyId::Tdfy,
        mint_amount: ONE_TDFY,
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
