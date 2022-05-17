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
use std::str::FromStr;
use tidefi_primitives::{
  pallet::SecurityExt, AssetId, ComplianceLevel, CurrencyId, Hash, Mint, ProposalType, Withdrawal,
};

type AccountId = u64;
type BlockNumber = u64;

const ASSET_1: AssetId = 1u32;
const ASSET_2: AssetId = 2u32;
const ALICE_ACCOUNT_ID: u32 = 1;
const BOB_ACCOUNT_ID: u32 = 2;
const ONE_TDFY: u128 = 1_000_000_000_000;
const BLOCK_NUMBER_ZERO: u64 = 0;

#[derive(Clone)]
struct UpdateConfiguration {
  members: Vec<AccountId>,
  threshold: u16,
}

#[derive(Clone)]
struct Context {
  alice: Origin,
  bob: Origin,
  pub_key: Vec<u8>,
  public_keys: BoundedVec<(AccountId, BoundedVec<u8, StringLimit>), PubkeyLimitPerAsset>,
  proposal_id: Hash,
  valid_mint: Mint<AccountId, Vec<u8>>,
  valid_withdrawal: Withdrawal<AccountId, BlockNumber, Vec<u8>>,
  valid_update_configuration: UpdateConfiguration,
  valid_mint_proposal: ProposalType<AccountId, BlockNumber, Vec<u8>, Vec<AccountId>>,
  valid_withdrawal_proposal: ProposalType<AccountId, BlockNumber, Vec<u8>, Vec<AccountId>>,
  valid_update_configuration_proposal:
    ProposalType<AccountId, BlockNumber, Vec<u8>, Vec<AccountId>>,
}

impl Default for Context {
  fn default() -> Self {
    let pub_key_bounded_vec: BoundedVec<u8, StringLimit> =
      "pubkey".as_bytes().to_vec().try_into().unwrap();
    let valid_mint = Mint {
      account_id: ALICE_ACCOUNT_ID.into(),
      currency_id: CurrencyId::Tdfy,
      mint_amount: ONE_TDFY,
      transaction_id: Vec::new(),
      compliance_level: ComplianceLevel::Green,
    };
    let valid_withdrawal = Withdrawal {
      account_id: ALICE_ACCOUNT_ID.into(),
      asset_id: CurrencyId::Tdfy,
      amount: ONE_TDFY,
      external_address: Vec::new(),
      block_number: BLOCK_NUMBER_ZERO,
    };
    let valid_update_configuration = UpdateConfiguration {
      members: vec![ALICE_ACCOUNT_ID.into(), BOB_ACCOUNT_ID.into()],
      threshold: 1u16,
    };

    Self {
      alice: Origin::signed(ALICE_ACCOUNT_ID.into()),
      bob: Origin::signed(BOB_ACCOUNT_ID.into()),
      pub_key: pub_key_bounded_vec.to_vec(),
      public_keys: vec![(ALICE_ACCOUNT_ID.into(), pub_key_bounded_vec)]
        .try_into()
        .unwrap(),
      proposal_id: Hash::from_str(
        "0x02a204a25c36f8c88eea76e08cdaa22a0569ef630bf4416db72abb9fb2445f2b",
      )
      .unwrap(),
      valid_mint: valid_mint.clone(),
      valid_withdrawal: valid_withdrawal.clone(),
      valid_update_configuration: valid_update_configuration.clone(),
      valid_mint_proposal: ProposalType::Mint(valid_mint.clone()),
      valid_withdrawal_proposal: ProposalType::Withdrawal(valid_withdrawal.clone()),
      valid_update_configuration_proposal: ProposalType::UpdateConfiguration(
        valid_update_configuration.members,
        valid_update_configuration.threshold,
      ),
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

  fn get_valid_proposals(
    &self,
  ) -> Vec<ProposalType<AccountId, BlockNumber, Vec<u8>, Vec<AccountId>>> {
    vec![
      ProposalType::Mint(self.valid_mint.clone()),
      ProposalType::Withdrawal(self.valid_withdrawal.clone()),
      ProposalType::UpdateConfiguration(
        self.valid_update_configuration.members.clone(),
        self.valid_update_configuration.threshold,
      ),
    ]
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

  mod succeeds_for {
    use super::*;

    #[test]
    pub fn mint() {
      new_test_ext().execute_with(|| {
        let context = Context::default().insert_asset1_with_alice_public_key();

        assert_ok!(Quorum::submit_proposal(
          context.alice.clone(),
          context.valid_mint_proposal
        ));

        assert_eq!(
          Quorum::proposals().into_inner().first().unwrap(),
          &(
            context.proposal_id,
            BLOCK_NUMBER_ZERO,
            ProposalType::Mint(Mint {
              account_id: context.valid_mint.account_id,
              currency_id: context.valid_mint.currency_id,
              mint_amount: context.valid_mint.mint_amount,
              transaction_id: BoundedVec::try_from(context.valid_mint.transaction_id).unwrap(),
              compliance_level: context.valid_mint.compliance_level,
            })
          )
        );
      });
    }

    #[test]
    pub fn withdrawal() {
      new_test_ext().execute_with(|| {
        let context = Context::default().insert_asset1_with_alice_public_key();

        assert_ok!(Quorum::submit_proposal(
          context.alice.clone(),
          context.valid_withdrawal_proposal
        ));

        assert_eq!(
          Quorum::proposals().into_inner().first().unwrap(),
          &(
            context.proposal_id,
            BLOCK_NUMBER_ZERO,
            ProposalType::Withdrawal(Withdrawal {
              account_id: context.valid_withdrawal.account_id,
              asset_id: context.valid_withdrawal.asset_id,
              amount: context.valid_withdrawal.amount,
              external_address: BoundedVec::try_from(context.valid_withdrawal.external_address)
                .unwrap(),
              block_number: context.valid_withdrawal.block_number,
            })
          )
        );
      });
    }

    #[test]
    pub fn update_configuration() {
      new_test_ext().execute_with(|| {
        let context = Context::default().insert_asset1_with_alice_public_key();

        assert_ok!(Quorum::submit_proposal(
          context.alice,
          context.valid_update_configuration_proposal
        ));

        assert_eq!(
          Quorum::proposals().into_inner().first().unwrap(),
          &(
            context.proposal_id,
            BLOCK_NUMBER_ZERO,
            ProposalType::UpdateConfiguration(
              BoundedVec::try_from(context.valid_update_configuration.members).unwrap(),
              context.valid_update_configuration.threshold
            )
          )
        );
      });
    }
  }

  mod fails_when {
    use super::*;

    #[test]
    pub fn sender_is_not_signed() {
      new_test_ext().execute_with(|| {
        let context = Context::default().insert_asset1_with_alice_public_key();

        for proposal in context.get_valid_proposals() {
          assert_noop!(Quorum::submit_proposal(Origin::none(), proposal), BadOrigin);
        }
      });
    }

    #[test]
    pub fn sender_is_not_a_member() {
      new_test_ext().execute_with(|| {
        let context = Context::default().insert_asset1_with_alice_public_key();

        for proposal in context.get_valid_proposals() {
          assert_noop!(
            Quorum::submit_proposal(context.bob.clone(), proposal),
            Error::<Test>::AccessDenied
          );
        }
      });
    }
  }
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

// Including tests for both acknowledge_proposal and reject_proposal
mod voting_for_proposals {
  use super::*;

  mod succeeds_when {
    use super::*;

    #[test]
    pub fn acknowledge_proposal() {
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
    pub fn reject_proposal() {
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
        assert_ok!(Quorum::reject_proposal(context.alice, context.proposal_id));
      });
    }
  }

  mod fails_when {
    use super::*;

    #[test]
    pub fn proposal_does_not_exist() {
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
    pub fn proposal_is_in_future_block() {
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
    pub fn proposal_is_expired() {
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
}
