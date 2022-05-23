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
    new_test_ext, Adapter, Event as MockEvent, Origin, ProposalLifetime, PubkeyLimitPerAsset,
    Quorum, Security, StringLimit, System, Test, VotesLimit, WatchListLimit,
  },
  pallet::*,
};
use frame_support::{
  assert_err, assert_noop, assert_ok,
  traits::{fungibles::Inspect, Hooks},
  BoundedVec,
};
use sp_core::H256;
use sp_runtime::traits::BadOrigin;
use std::str::FromStr;

use pallet_security::CurrentBlockCount as CurrentBlockNumber;
use tidefi_primitives::{
  pallet::SecurityExt, AssetId, ComplianceLevel, CurrencyId, Hash, Mint, ProposalStatus,
  ProposalType, ProposalVotes, WatchList, WatchListAction, Withdrawal,
};

type AccountId = u64;
type BlockNumber = u64;

const ASSET_1: AssetId = 1u32;
const ALICE_ACCOUNT_ID: u32 = 1;
const BOB_ACCOUNT_ID: u32 = 2;
const BLOCK_NUMBER_ZERO: u64 = 0;

// TEMP Asset
const TEMP_ASSET_ID: AssetId = 4;

// Asset Units
const ONE_TEMP: u128 = 100;
const ONE_TDFY: u128 = 1_000_000_000_000;

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
      asset_id: CurrencyId::Wrapped(TEMP_ASSET_ID),
      amount: ONE_TEMP,
      external_address: Vec::new(),
      block_number: BLOCK_NUMBER_ZERO,
    };
    let valid_update_configuration = UpdateConfiguration {
      members: vec![ALICE_ACCOUNT_ID.into(), BOB_ACCOUNT_ID.into()],
      threshold: 2u16,
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

  fn insert_a_valid_mint_proposal_with_green_compliance_level(self) -> Self {
    insert_mint_proposal(Default::default(), ComplianceLevel::Green, self.proposal_id);
    self
  }

  fn insert_a_valid_mint_proposal_with_amber_compliance_level(self) -> Self {
    insert_mint_proposal(Default::default(), ComplianceLevel::Amber, self.proposal_id);
    self
  }

  fn insert_a_valid_mint_proposal_with_red_compliance_level(self) -> Self {
    insert_mint_proposal(Default::default(), ComplianceLevel::Red, self.proposal_id);
    self
  }

  fn insert_a_valid_withdrawal_proposal(self) -> Self {
    let proposal = ProposalType::Withdrawal(Withdrawal {
      account_id: self.valid_withdrawal.account_id,
      asset_id: self.valid_withdrawal.asset_id,
      amount: self.valid_withdrawal.amount,
      external_address: BoundedVec::try_from(self.valid_withdrawal.external_address.clone())
        .unwrap(),
      block_number: self.valid_withdrawal.block_number,
    });
    assert_ok!(Proposals::<Test>::try_append((
      self.proposal_id,
      Security::get_current_block_count(),
      proposal
    )));
    self
  }

  fn insert_a_valid_update_configuration_proposal(self) -> Self {
    let proposal = ProposalType::UpdateConfiguration(
      BoundedVec::try_from(self.valid_update_configuration.members.clone()).unwrap(),
      self.valid_update_configuration.threshold,
    );
    assert_ok!(Proposals::<Test>::try_append((
      self.proposal_id,
      Security::get_current_block_count(),
      proposal
    )));
    self
  }

  fn set_valid_proposal_status(self, status: ProposalStatus) -> Self {
    let mut votes = ProposalVotes::default();
    votes.status = status;
    Votes::<Test>::insert(self.proposal_id, votes);
    self
  }

  fn set_vote_for_alice(self, is_acknowledge: bool) -> Self {
    let mut votes = ProposalVotes::default();
    match is_acknowledge {
      true => votes.votes_for = BoundedVec::try_from(vec![ALICE_ACCOUNT_ID as u64]).unwrap(),
      false => votes.votes_against = BoundedVec::try_from(vec![ALICE_ACCOUNT_ID as u64]).unwrap(),
    }
    Votes::<Test>::insert(self.proposal_id, votes);
    self
  }

  fn set_multiple_dummy_votes(self, number_of_mock_votes: u32, is_acknowledge: bool) -> Self {
    let mut votes = ProposalVotes::default();

    match is_acknowledge {
      true => {
        votes.votes_for = BoundedVec::try_from(vec![
          BOB_ACCOUNT_ID as u64;
          usize::try_from(number_of_mock_votes).unwrap()
        ])
        .unwrap()
      }
      false => {
        votes.votes_against = BoundedVec::try_from(vec![
          BOB_ACCOUNT_ID as u64;
          usize::try_from(number_of_mock_votes)
            .unwrap()
        ])
        .unwrap()
      }
    }
    Votes::<Test>::insert(self.proposal_id, votes);
    self
  }

  fn set_threshold(self, threshold: u16) -> Self {
    Threshold::<Test>::put(threshold);
    self
  }

  fn commit_a_valid_vote(self, is_acknowledge: bool) -> Self {
    let mut votes = ProposalVotes::<BlockNumber, BoundedVec<AccountId, VotesLimit>>::default();
    votes.expiry = BLOCK_NUMBER_ZERO + ProposalLifetime::get();
    match is_acknowledge {
      true => {
        votes.votes_for = BoundedVec::try_from(vec![ALICE_ACCOUNT_ID as u64]).unwrap();
      }
      false => {
        votes.votes_against = BoundedVec::try_from(vec![ALICE_ACCOUNT_ID as u64]).unwrap();
      }
    }
    Votes::<Test>::insert(self.proposal_id, votes);
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

fn insert_mint_proposal(
  transaction_id: BoundedVec<u8, StringLimit>,
  compliance_level: ComplianceLevel,
  proposal_id: H256,
) {
  let proposal = ProposalType::Mint(Mint {
    account_id: ALICE_ACCOUNT_ID.into(),
    currency_id: CurrencyId::Tdfy,
    mint_amount: ONE_TDFY,
    transaction_id: transaction_id,
    compliance_level: compliance_level,
  });
  assert_ok!(Proposals::<Test>::try_append((
    proposal_id,
    Security::get_current_block_count(),
    proposal
  )));
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

        System::assert_has_event(MockEvent::Quorum(Event::ProposalSubmitted {
          proposal_id: context.proposal_id,
        }));
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

        System::assert_has_event(MockEvent::Quorum(Event::ProposalSubmitted {
          proposal_id: context.proposal_id,
        }));
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

        System::assert_has_event(MockEvent::Quorum(Event::ProposalSubmitted {
          proposal_id: context.proposal_id,
        }));
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

    #[ignore]
    #[test]
    pub fn mint_transaction_id_is_bad() {
      new_test_ext().execute_with(|| {
        let context = Context::default().insert_asset1_with_alice_public_key();
        let proposal = ProposalType::Mint(Mint {
          account_id: context.valid_mint.account_id,
          currency_id: context.valid_mint.currency_id,
          mint_amount: context.valid_mint.mint_amount,
          transaction_id: vec![0; u32::MAX.try_into().unwrap()],
          compliance_level: context.valid_mint.compliance_level,
        });

        assert_noop!(
          Quorum::submit_proposal(context.alice, proposal),
          Error::<Test>::BadTransactionId
        );
      });
    }

    #[ignore]
    #[test]
    pub fn withdrawal_external_address_is_bad() {
      new_test_ext().execute_with(|| {
        let context = Context::default().insert_asset1_with_alice_public_key();
        let proposal = ProposalType::Withdrawal(Withdrawal {
          account_id: context.valid_withdrawal.account_id,
          asset_id: context.valid_withdrawal.asset_id,
          amount: context.valid_withdrawal.amount,
          external_address: vec![0; u32::MAX.try_into().unwrap()],
          block_number: BLOCK_NUMBER_ZERO,
        });

        assert_noop!(
          Quorum::submit_proposal(context.alice, proposal),
          Error::<Test>::BadExternalAddress
        );
      });
    }

    #[ignore]
    #[test]
    pub fn update_configuration_with_members_overflow() {
      new_test_ext().execute_with(|| {
        let context = Context::default().insert_asset1_with_alice_public_key();
        let proposal = ProposalType::UpdateConfiguration(
          vec![0u64; u16::MAX as usize],
          context.valid_update_configuration.threshold,
        );

        assert_noop!(
          Quorum::submit_proposal(context.alice, proposal),
          Error::<Test>::MembersOverflow
        );
      })
    }
  }
}

// Including tests for both acknowledge_proposal and reject_proposal
mod voting_for_proposals {
  use super::*;

  mod mint_proposal_is_successfully_committed {
    use super::*;

    mod and_resolved {
      use super::*;

      #[test]
      pub fn acknowledged_with_compliance_level_green() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_mint_proposal_with_green_compliance_level();

          let asset_balance_before_mint = Adapter::balance(
            context.valid_mint.currency_id,
            &context.valid_mint.account_id,
          );

          assert_ok!(Quorum::acknowledge_proposal(
            context.alice,
            context.proposal_id
          ));

          assert!(Quorum::proposals()
            .iter()
            .find(|&&(proposal_id, _, _)| proposal_id == context.proposal_id)
            .is_none());

          assert!(Quorum::proposal_votes(context.proposal_id).is_none());

          assert_eq!(
            asset_balance_before_mint + context.valid_mint.mint_amount,
            Adapter::balance(
              context.valid_mint.currency_id,
              &context.valid_mint.account_id,
            )
          );

          System::assert_has_event(MockEvent::Quorum(Event::VoteFor {
            account_id: ALICE_ACCOUNT_ID as u64,
            proposal_id: context.proposal_id,
          }));

          System::assert_has_event(MockEvent::Quorum(Event::ProposalApproved {
            proposal_id: context.proposal_id,
          }));

          System::assert_has_event(MockEvent::Quorum(Event::Minted {
            proposal_id: context.proposal_id,
            account_id: context.valid_mint.account_id,
            currency_id: context.valid_mint.currency_id,
            amount: context.valid_mint.mint_amount,
            transaction_id: context.valid_mint.transaction_id,
            compliance_level: context.valid_mint.compliance_level,
          }));

          System::assert_has_event(MockEvent::Quorum(Event::ProposalProcessed {
            proposal_id: context.proposal_id,
          }));
        });
      }

      #[test]
      pub fn acknowledged_with_compliance_level_amber() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_mint_proposal_with_amber_compliance_level();

          let asset_balance_before_mint = Adapter::balance(
            context.valid_mint.currency_id,
            &context.valid_mint.account_id,
          );

          assert_ok!(Quorum::acknowledge_proposal(
            context.alice,
            context.proposal_id
          ));

          assert!(Quorum::proposals()
            .iter()
            .find(|&&(proposal_id, _, _)| proposal_id == context.proposal_id)
            .is_none());

          assert!(Quorum::proposal_votes(context.proposal_id).is_none());

          assert_eq!(
            asset_balance_before_mint + context.valid_mint.mint_amount,
            Adapter::balance(
              context.valid_mint.currency_id,
              &context.valid_mint.account_id,
            )
          );

          System::assert_has_event(MockEvent::Quorum(Event::VoteFor {
            account_id: ALICE_ACCOUNT_ID as u64,
            proposal_id: context.proposal_id,
          }));

          System::assert_has_event(MockEvent::Quorum(Event::ProposalApproved {
            proposal_id: context.proposal_id,
          }));

          System::assert_has_event(MockEvent::Quorum(Event::Minted {
            proposal_id: context.proposal_id,
            account_id: context.valid_mint.account_id,
            currency_id: context.valid_mint.currency_id,
            amount: context.valid_mint.mint_amount,
            transaction_id: context.valid_mint.transaction_id.clone(),
            compliance_level: ComplianceLevel::Amber,
          }));

          System::assert_has_event(MockEvent::Quorum(Event::ProposalProcessed {
            proposal_id: context.proposal_id,
          }));

          assert_eq!(
            Quorum::account_watch_list(ALICE_ACCOUNT_ID as u64)
              .unwrap()
              .into_inner()
              .first()
              .unwrap(),
            &WatchList {
              amount: context.valid_mint.mint_amount,
              block_number: BLOCK_NUMBER_ZERO,
              compliance_level: ComplianceLevel::Amber,
              currency_id: context.valid_mint.currency_id,
              watch_action: WatchListAction::Mint,
              transaction_id: BoundedVec::try_from(context.valid_mint.transaction_id.clone())
                .unwrap(),
            }
          );

          System::assert_has_event(MockEvent::Quorum(Event::WatchTransactionAdded {
            account_id: context.valid_mint.account_id,
            currency_id: context.valid_mint.currency_id,
            amount: context.valid_mint.mint_amount,
            compliance_level: ComplianceLevel::Amber,
            watch_action: WatchListAction::Mint,
            transaction_id: context.valid_mint.transaction_id,
          }));
        });
      }

      #[test]
      pub fn acknowledged_with_compliance_level_red() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_mint_proposal_with_red_compliance_level();

          assert_ok!(Quorum::acknowledge_proposal(
            context.alice,
            context.proposal_id
          ));

          assert!(Quorum::proposals()
            .iter()
            .find(|&&(proposal_id, _, _)| proposal_id == context.proposal_id)
            .is_none());

          assert!(Quorum::proposal_votes(context.proposal_id).is_none());

          System::assert_has_event(MockEvent::Quorum(Event::VoteFor {
            account_id: ALICE_ACCOUNT_ID as u64,
            proposal_id: context.proposal_id,
          }));

          System::assert_has_event(MockEvent::Quorum(Event::ProposalApproved {
            proposal_id: context.proposal_id,
          }));

          System::assert_has_event(MockEvent::Quorum(Event::ProposalProcessed {
            proposal_id: context.proposal_id,
          }));

          assert_eq!(
            Quorum::account_watch_list(ALICE_ACCOUNT_ID as u64)
              .unwrap()
              .into_inner()
              .first()
              .unwrap(),
            &WatchList {
              amount: context.valid_mint.mint_amount,
              block_number: BLOCK_NUMBER_ZERO,
              compliance_level: ComplianceLevel::Red,
              currency_id: context.valid_mint.currency_id,
              watch_action: WatchListAction::Mint,
              transaction_id: BoundedVec::try_from(context.valid_mint.transaction_id.clone())
                .unwrap(),
            }
          );

          System::assert_has_event(MockEvent::Quorum(Event::WatchTransactionAdded {
            account_id: context.valid_mint.account_id,
            currency_id: context.valid_mint.currency_id,
            amount: context.valid_mint.mint_amount,
            compliance_level: ComplianceLevel::Red,
            watch_action: WatchListAction::Mint,
            transaction_id: context.valid_mint.transaction_id,
          }));
        });
      }

      #[test]
      pub fn rejected() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_mint_proposal_with_green_compliance_level();

          let asset_balance_before_mint = Adapter::balance(
            context.valid_mint.currency_id,
            &context.valid_mint.account_id,
          );

          assert_ok!(Quorum::reject_proposal(context.alice, context.proposal_id));

          assert!(Quorum::proposals()
            .iter()
            .find(|&&(proposal_id, _, _)| proposal_id == context.proposal_id)
            .is_none());

          assert!(Quorum::proposal_votes(context.proposal_id).is_none());

          assert_eq!(
            asset_balance_before_mint,
            Adapter::balance(
              context.valid_mint.currency_id,
              &context.valid_mint.account_id,
            )
          );

          System::assert_has_event(MockEvent::Quorum(Event::VoteAgainst {
            account_id: ALICE_ACCOUNT_ID as u64,
            proposal_id: context.proposal_id,
          }));

          System::assert_has_event(MockEvent::Quorum(Event::ProposalRejected {
            proposal_id: context.proposal_id,
          }));
        });
      }

      #[test]
      pub fn and_added_a_new_record_to_the_account_watch_list() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_mint_proposal_with_amber_compliance_level();

          assert_ok!(Quorum::acknowledge_proposal(
            context.alice.clone(),
            context.proposal_id
          ));

          insert_mint_proposal(
            Default::default(),
            ComplianceLevel::Amber,
            context.proposal_id,
          );

          assert_ok!(Quorum::acknowledge_proposal(
            context.alice,
            context.proposal_id
          ));

          assert_eq!(
            Quorum::account_watch_list(ALICE_ACCOUNT_ID as u64)
              .unwrap()
              .len(),
            2
          );

          assert_eq!(
            Quorum::account_watch_list(ALICE_ACCOUNT_ID as u64)
              .unwrap()
              .into_inner()
              .first()
              .unwrap(),
            &WatchList {
              amount: context.valid_mint.mint_amount,
              block_number: BLOCK_NUMBER_ZERO,
              compliance_level: ComplianceLevel::Amber,
              currency_id: context.valid_mint.currency_id,
              watch_action: WatchListAction::Mint,
              transaction_id: BoundedVec::try_from(context.valid_mint.transaction_id.clone())
                .unwrap(),
            }
          );

          assert_eq!(
            Quorum::account_watch_list(ALICE_ACCOUNT_ID as u64)
              .unwrap()
              .into_inner()[1],
            WatchList {
              amount: context.valid_mint.mint_amount,
              block_number: BLOCK_NUMBER_ZERO,
              compliance_level: ComplianceLevel::Amber,
              currency_id: context.valid_mint.currency_id,
              watch_action: WatchListAction::Mint,
              transaction_id: BoundedVec::try_from(context.valid_mint.transaction_id.clone())
                .unwrap(),
            }
          );
        });
      }
    }

    mod but_not_resolved {
      use super::*;

      #[test]
      pub fn acknowledged() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_mint_proposal_with_green_compliance_level()
            .set_threshold(2);

          let asset_balance_before_mint = Adapter::balance(
            context.valid_mint.currency_id,
            &context.valid_mint.account_id,
          );

          assert_ok!(Quorum::acknowledge_proposal(
            context.alice,
            context.proposal_id
          ));

          assert_eq!(
            asset_balance_before_mint,
            Adapter::balance(
              context.valid_mint.currency_id,
              &context.valid_mint.account_id,
            )
          );

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

          assert!(Quorum::proposal_votes(context.proposal_id)
            .unwrap()
            .votes_for
            .into_inner()
            .contains(&(ALICE_ACCOUNT_ID as u64)));

          System::assert_has_event(MockEvent::Quorum(Event::VoteFor {
            account_id: ALICE_ACCOUNT_ID as u64,
            proposal_id: context.proposal_id,
          }));
        });
      }

      #[test]
      pub fn rejected() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_mint_proposal_with_green_compliance_level()
            .set_threshold(2);

          assert_ok!(Quorum::reject_proposal(context.alice, context.proposal_id));

          assert!(Quorum::proposal_votes(context.proposal_id)
            .unwrap()
            .votes_against
            .into_inner()
            .contains(&(ALICE_ACCOUNT_ID as u64)));

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

          assert!(Quorum::proposal_votes(context.proposal_id)
            .unwrap()
            .votes_against
            .into_inner()
            .contains(&(ALICE_ACCOUNT_ID as u64)));

          System::assert_has_event(MockEvent::Quorum(Event::VoteAgainst {
            account_id: ALICE_ACCOUNT_ID as u64,
            proposal_id: context.proposal_id,
          }));
        });
      }
    }
  }

  mod update_configuration_proposal_is_successfully_committed {
    use super::*;

    mod and_resolved {
      use super::*;

      #[test]
      pub fn acknowledged() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_update_configuration_proposal();

          assert_eq!(true, Quorum::members(ALICE_ACCOUNT_ID as u64).unwrap());
          assert!(Quorum::members(BOB_ACCOUNT_ID as u64).is_none());
          let threshold_before = Quorum::threshold();

          assert_ok!(Quorum::acknowledge_proposal(
            context.alice,
            context.proposal_id
          ));

          assert!(Quorum::proposals()
            .iter()
            .find(|&&(proposal_id, _, _)| proposal_id == context.proposal_id)
            .is_none());

          assert!(Quorum::proposal_votes(context.proposal_id).is_none());

          assert_eq!(true, Quorum::members(ALICE_ACCOUNT_ID as u64).unwrap());
          assert_eq!(true, Quorum::members(BOB_ACCOUNT_ID as u64).unwrap());
          assert!(threshold_before != Quorum::threshold());
          assert_eq!(
            context.valid_update_configuration.threshold,
            Quorum::threshold()
          );

          System::assert_has_event(MockEvent::Quorum(Event::VoteFor {
            account_id: ALICE_ACCOUNT_ID as u64,
            proposal_id: context.proposal_id,
          }));

          System::assert_has_event(MockEvent::Quorum(Event::ProposalApproved {
            proposal_id: context.proposal_id,
          }));

          System::assert_has_event(MockEvent::Quorum(Event::ConfigurationUpdated {
            threshold: context.valid_update_configuration.threshold,
            members: context.valid_update_configuration.members,
          }));

          System::assert_has_event(MockEvent::Quorum(Event::ProposalProcessed {
            proposal_id: context.proposal_id,
          }));
        });
      }

      #[test]
      pub fn rejected() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_mint_proposal_with_green_compliance_level();

          assert_eq!(true, Quorum::members(ALICE_ACCOUNT_ID as u64).unwrap());
          assert!(Quorum::members(BOB_ACCOUNT_ID as u64).is_none());
          let threshold_before = Quorum::threshold();

          assert_ok!(Quorum::reject_proposal(context.alice, context.proposal_id));

          assert!(Quorum::proposals()
            .iter()
            .find(|&&(proposal_id, _, _)| proposal_id == context.proposal_id)
            .is_none());

          assert!(Quorum::proposal_votes(context.proposal_id).is_none());

          assert_eq!(true, Quorum::members(ALICE_ACCOUNT_ID as u64).unwrap());
          assert!(Quorum::members(BOB_ACCOUNT_ID as u64).is_none());
          assert!(threshold_before == Quorum::threshold());

          System::assert_has_event(MockEvent::Quorum(Event::VoteAgainst {
            account_id: ALICE_ACCOUNT_ID as u64,
            proposal_id: context.proposal_id,
          }));

          System::assert_has_event(MockEvent::Quorum(Event::ProposalRejected {
            proposal_id: context.proposal_id,
          }));
        });
      }
    }

    mod but_not_resolved {
      use super::*;

      #[test]
      pub fn acknowledged() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_update_configuration_proposal()
            .set_threshold(2);

          assert_eq!(true, Quorum::members(ALICE_ACCOUNT_ID as u64).unwrap());
          assert!(Quorum::members(BOB_ACCOUNT_ID as u64).is_none());
          let threshold_before = Quorum::threshold();

          assert_ok!(Quorum::acknowledge_proposal(
            context.alice,
            context.proposal_id
          ));

          assert_eq!(true, Quorum::members(ALICE_ACCOUNT_ID as u64).unwrap());
          assert!(Quorum::members(BOB_ACCOUNT_ID as u64).is_none());
          assert!(threshold_before == Quorum::threshold());

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

          assert!(Quorum::proposal_votes(context.proposal_id)
            .unwrap()
            .votes_for
            .into_inner()
            .contains(&(ALICE_ACCOUNT_ID as u64)));

          System::assert_has_event(MockEvent::Quorum(Event::VoteFor {
            account_id: ALICE_ACCOUNT_ID as u64,
            proposal_id: context.proposal_id,
          }));
        });
      }

      #[test]
      pub fn rejected() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_update_configuration_proposal()
            .set_threshold(2);

          assert_eq!(true, Quorum::members(ALICE_ACCOUNT_ID as u64).unwrap());
          assert!(Quorum::members(BOB_ACCOUNT_ID as u64).is_none());
          let threshold_before = Quorum::threshold();

          assert_ok!(Quorum::reject_proposal(context.alice, context.proposal_id));

          assert_eq!(true, Quorum::members(ALICE_ACCOUNT_ID as u64).unwrap());
          assert!(Quorum::members(BOB_ACCOUNT_ID as u64).is_none());
          assert!(threshold_before == Quorum::threshold());

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

          assert!(Quorum::proposal_votes(context.proposal_id)
            .unwrap()
            .votes_against
            .into_inner()
            .contains(&(ALICE_ACCOUNT_ID as u64)));

          System::assert_has_event(MockEvent::Quorum(Event::VoteAgainst {
            account_id: ALICE_ACCOUNT_ID as u64,
            proposal_id: context.proposal_id,
          }));
        });
      }
    }
  }

  mod fails_when {
    use super::*;

    mod commit_vote {
      use super::*;

      #[test]
      pub fn sender_is_not_signed() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_mint_proposal_with_green_compliance_level();

          assert_noop!(
            Quorum::acknowledge_proposal(Origin::none(), context.proposal_id),
            BadOrigin
          );
          assert_noop!(
            Quorum::reject_proposal(Origin::none(), context.proposal_id),
            BadOrigin
          );
        });
      }

      #[test]
      pub fn sender_is_not_a_member() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_mint_proposal_with_green_compliance_level();

          assert_noop!(
            Quorum::acknowledge_proposal(context.bob.clone(), context.proposal_id),
            Error::<Test>::AccessDenied
          );
          assert_noop!(
            Quorum::reject_proposal(context.bob, context.proposal_id),
            Error::<Test>::AccessDenied
          );
        });
      }

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
      pub fn proposal_block_is_in_future() {
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
      pub fn proposal_is_already_completed() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_mint_proposal_with_green_compliance_level()
            .set_valid_proposal_status(ProposalStatus::Approved);

          assert_noop!(
            Quorum::acknowledge_proposal(context.alice.clone(), context.proposal_id),
            Error::<Test>::ProposalAlreadyComplete
          );
          assert_noop!(
            Quorum::reject_proposal(context.alice, context.proposal_id),
            Error::<Test>::ProposalAlreadyComplete
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

      #[test]
      pub fn member_already_voted() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_mint_proposal_with_green_compliance_level()
            .set_vote_for_alice(true);

          assert_noop!(
            Quorum::acknowledge_proposal(context.alice.clone(), context.proposal_id),
            Error::<Test>::MemberAlreadyVoted
          );

          let context = Context::default().set_vote_for_alice(false);
          assert_noop!(
            Quorum::reject_proposal(context.alice, context.proposal_id),
            Error::<Test>::MemberAlreadyVoted
          );
        });
      }

      #[test]
      pub fn votes_for_overflow() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_mint_proposal_with_green_compliance_level()
            .set_multiple_dummy_votes(VotesLimit::get(), true);

          assert_noop!(
            Quorum::acknowledge_proposal(context.alice.clone(), context.proposal_id),
            Error::<Test>::VotesForOverflow
          );
        });
      }

      #[test]
      pub fn votes_against_overflow() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .insert_asset1_with_alice_public_key()
            .insert_a_valid_mint_proposal_with_green_compliance_level()
            .set_multiple_dummy_votes(VotesLimit::get(), false);

          assert_noop!(
            Quorum::reject_proposal(context.alice, context.proposal_id),
            Error::<Test>::VotesAgainstOverflow
          );
        });
      }
    }

    mod try_resolve_proposal {
      use super::*;

      mod mint {
        use super::*;

        #[test]
        pub fn asset_is_disabled() {
          new_test_ext().execute_with(|| {
            let context = Context::default().insert_asset1_with_alice_public_key();

            let disabled_asset_id = CurrencyId::Wrapped(TEMP_ASSET_ID);

            let proposal = ProposalType::Mint(Mint {
              account_id: ALICE_ACCOUNT_ID.into(),
              currency_id: disabled_asset_id,
              mint_amount: ONE_TEMP,
              transaction_id: BoundedVec::try_from(Vec::new()).unwrap(),
              compliance_level: ComplianceLevel::Green,
            });
            assert_ok!(Proposals::<Test>::try_append((
              context.proposal_id,
              Security::get_current_block_count(),
              proposal
            )));

            let temp_asset_balance_before_mint =
              Adapter::balance(disabled_asset_id, &context.valid_mint.account_id);

            assert_err!(
              Quorum::acknowledge_proposal(context.alice, context.proposal_id),
              Error::<Test>::AssetDisabled
            );

            assert_eq!(
              Quorum::proposals().into_inner().first().unwrap(),
              &(
                context.proposal_id,
                BLOCK_NUMBER_ZERO,
                ProposalType::Mint(Mint {
                  account_id: context.valid_mint.account_id,
                  currency_id: disabled_asset_id,
                  mint_amount: ONE_TEMP,
                  transaction_id: BoundedVec::try_from(context.valid_mint.transaction_id.clone())
                    .unwrap(),
                  compliance_level: ComplianceLevel::Green,
                })
              )
            );

            assert!(Quorum::proposal_votes(context.proposal_id)
              .unwrap()
              .votes_for
              .into_inner()
              .contains(&(ALICE_ACCOUNT_ID as u64)));

            assert_eq!(
              temp_asset_balance_before_mint,
              Adapter::balance(disabled_asset_id, &context.valid_mint.account_id,)
            );

            System::assert_has_event(MockEvent::Quorum(Event::VoteFor {
              account_id: ALICE_ACCOUNT_ID as u64,
              proposal_id: context.proposal_id,
            }));

            System::assert_has_event(MockEvent::Quorum(Event::ProposalApproved {
              proposal_id: context.proposal_id,
            }));
          });
        }

        #[test]
        pub fn watch_list_overflow() {
          new_test_ext().execute_with(|| {
            let context = Context::default()
              .insert_asset1_with_alice_public_key()
              .insert_a_valid_mint_proposal_with_amber_compliance_level();

            let asset_balance_before_mint = Adapter::balance(
              context.valid_mint.currency_id,
              &context.valid_mint.account_id,
            );

            let watch_list = WatchList {
              amount: context.valid_mint.mint_amount,
              block_number: BLOCK_NUMBER_ZERO,
              compliance_level: ComplianceLevel::Amber,
              currency_id: context.valid_mint.currency_id,
              watch_action: WatchListAction::Mint,
              transaction_id: BoundedVec::try_from(context.valid_mint.transaction_id.clone())
                .unwrap(),
            };

            let max_watch_list: BoundedVec<
              WatchList<BlockNumber, BoundedVec<u8, StringLimit>>,
              WatchListLimit,
            > = vec![watch_list; WatchListLimit::get().try_into().unwrap()]
              .try_into()
              .expect("Valid watch list");
            AccountWatchList::<Test>::insert(context.valid_mint.account_id, max_watch_list);

            assert_err!(
              Quorum::acknowledge_proposal(context.alice, context.proposal_id),
              Error::<Test>::WatchlistOverflow
            );

            assert_eq!(
              Quorum::proposals().into_inner().first().unwrap(),
              &(
                context.proposal_id,
                BLOCK_NUMBER_ZERO,
                ProposalType::Mint(Mint {
                  account_id: context.valid_mint.account_id,
                  currency_id: context.valid_mint.currency_id,
                  mint_amount: context.valid_mint.mint_amount,
                  transaction_id: BoundedVec::try_from(context.valid_mint.transaction_id.clone())
                    .unwrap(),
                  compliance_level: ComplianceLevel::Amber,
                })
              )
            );

            assert!(Quorum::proposal_votes(context.proposal_id)
              .unwrap()
              .votes_for
              .into_inner()
              .contains(&(ALICE_ACCOUNT_ID as u64)));

            assert_eq!(
              asset_balance_before_mint,
              Adapter::balance(
                context.valid_mint.currency_id,
                &context.valid_mint.account_id,
              )
            );

            System::assert_has_event(MockEvent::Quorum(Event::VoteFor {
              account_id: ALICE_ACCOUNT_ID as u64,
              proposal_id: context.proposal_id,
            }));

            System::assert_has_event(MockEvent::Quorum(Event::ProposalApproved {
              proposal_id: context.proposal_id,
            }));
          });
        }
      }

      mod withdrawal {
        use super::*;

        #[test]
        pub fn asset_is_disabled() {
          new_test_ext().execute_with(|| {
            let context = Context::default()
              .insert_asset1_with_alice_public_key()
              .insert_a_valid_withdrawal_proposal();

            let temp_asset_balance_before_mint = Adapter::balance(
              context.valid_withdrawal.asset_id,
              &context.valid_withdrawal.account_id,
            );

            assert_err!(
              Quorum::acknowledge_proposal(context.alice, context.proposal_id),
              Error::<Test>::AssetDisabled
            );

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

            assert!(Quorum::proposal_votes(context.proposal_id)
              .unwrap()
              .votes_for
              .into_inner()
              .contains(&(ALICE_ACCOUNT_ID as u64)));

            assert_eq!(
              temp_asset_balance_before_mint,
              Adapter::balance(
                context.valid_withdrawal.asset_id,
                &context.valid_withdrawal.account_id,
              )
            );

            System::assert_has_event(MockEvent::Quorum(Event::VoteFor {
              account_id: ALICE_ACCOUNT_ID as u64,
              proposal_id: context.proposal_id,
            }));

            System::assert_has_event(MockEvent::Quorum(Event::ProposalApproved {
              proposal_id: context.proposal_id,
            }));
          });
        }
      }
    }
  }
}

mod acknowledge_burned {
  use super::*;

  #[test]
  pub fn succeeds() {
    new_test_ext().execute_with(|| {
      let context = Context::default()
        .insert_asset1_with_alice_public_key()
        .insert_a_valid_mint_proposal_with_green_compliance_level()
        .set_threshold(2);

      assert_ok!(Quorum::reject_proposal(
        context.alice.clone(),
        context.proposal_id
      ));

      assert_ok!(Quorum::acknowledge_burned(
        context.alice,
        context.proposal_id
      ));

      assert!(Quorum::burned_queue()
        .iter()
        .find(|&&(proposal_id, _)| proposal_id == context.proposal_id)
        .is_none());

      System::assert_has_event(MockEvent::Quorum(Event::BurnedAcknowledged {
        proposal_id: context.proposal_id,
      }));
    });
  }

  mod fails_when {
    use super::*;

    #[test]
    pub fn sender_is_not_signed() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .insert_asset1_with_alice_public_key()
          .insert_a_valid_mint_proposal_with_green_compliance_level()
          .set_threshold(2);

        assert_ok!(Quorum::reject_proposal(
          context.alice.clone(),
          context.proposal_id
        ));

        assert_noop!(
          Quorum::acknowledge_burned(Origin::none(), context.proposal_id),
          BadOrigin
        );
      });
    }

    #[test]
    pub fn sender_is_not_a_member() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .insert_asset1_with_alice_public_key()
          .insert_a_valid_mint_proposal_with_green_compliance_level()
          .set_threshold(2);

        assert_ok!(Quorum::reject_proposal(
          context.alice.clone(),
          context.proposal_id
        ));

        assert_noop!(
          Quorum::acknowledge_burned(context.bob, context.proposal_id),
          Error::<Test>::AccessDenied
        );
      });
    }
  }
}

mod eval_proposal_state {
  use super::*;

  mod mint_proposal_is_successfully {
    use super::*;

    #[test]
    pub fn initiated() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .insert_asset1_with_alice_public_key()
          .insert_a_valid_mint_proposal_with_green_compliance_level()
          .set_threshold(2)
          .commit_a_valid_vote(true);

        let asset_balance_before_mint = Adapter::balance(
          context.valid_mint.currency_id,
          &context.valid_mint.account_id,
        );

        assert_ok!(Quorum::eval_proposal_state(
          context.alice,
          context.proposal_id
        ));

        assert!(Quorum::proposals()
          .iter()
          .find(|&&(proposal_id, _, _)| proposal_id == context.proposal_id)
          .is_some());

        assert_eq!(
          Quorum::proposal_votes(context.proposal_id).unwrap().status,
          ProposalStatus::Initiated
        );

        assert!(Quorum::proposal_votes(context.proposal_id)
          .unwrap()
          .votes_for
          .into_inner()
          .contains(&(ALICE_ACCOUNT_ID as u64)));

        assert_eq!(
          asset_balance_before_mint,
          Adapter::balance(
            context.valid_mint.currency_id,
            &context.valid_mint.account_id,
          )
        );
      });
    }

    #[test]
    pub fn approved() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .insert_asset1_with_alice_public_key()
          .insert_a_valid_mint_proposal_with_green_compliance_level()
          .commit_a_valid_vote(true);

        let asset_balance_before_mint = Adapter::balance(
          context.valid_mint.currency_id,
          &context.valid_mint.account_id,
        );

        assert_ok!(Quorum::eval_proposal_state(
          context.alice,
          context.proposal_id
        ));

        assert!(Quorum::proposals()
          .iter()
          .find(|&&(proposal_id, _, _)| proposal_id == context.proposal_id)
          .is_none());

        assert!(Quorum::proposal_votes(context.proposal_id).is_none());

        assert_eq!(
          asset_balance_before_mint + context.valid_mint.mint_amount,
          Adapter::balance(
            context.valid_mint.currency_id,
            &context.valid_mint.account_id,
          )
        );

        System::assert_has_event(MockEvent::Quorum(Event::ProposalApproved {
          proposal_id: context.proposal_id,
        }));

        System::assert_has_event(MockEvent::Quorum(Event::Minted {
          proposal_id: context.proposal_id,
          account_id: context.valid_mint.account_id,
          currency_id: context.valid_mint.currency_id,
          amount: context.valid_mint.mint_amount,
          transaction_id: context.valid_mint.transaction_id,
          compliance_level: context.valid_mint.compliance_level,
        }));

        System::assert_has_event(MockEvent::Quorum(Event::ProposalProcessed {
          proposal_id: context.proposal_id,
        }));
      });
    }

    #[test]
    pub fn rejected() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .insert_asset1_with_alice_public_key()
          .insert_a_valid_mint_proposal_with_green_compliance_level()
          .commit_a_valid_vote(false);

        let asset_balance_before_mint = Adapter::balance(
          context.valid_mint.currency_id,
          &context.valid_mint.account_id,
        );

        assert_ok!(Quorum::eval_proposal_state(
          context.alice,
          context.proposal_id
        ));

        assert!(Quorum::proposals()
          .iter()
          .find(|&&(proposal_id, _, _)| proposal_id == context.proposal_id)
          .is_none());

        assert!(Quorum::proposal_votes(context.proposal_id).is_none());

        assert_eq!(
          asset_balance_before_mint,
          Adapter::balance(
            context.valid_mint.currency_id,
            &context.valid_mint.account_id,
          )
        );

        System::assert_has_event(MockEvent::Quorum(Event::ProposalRejected {
          proposal_id: context.proposal_id,
        }));
      });
    }
  }

  mod withdrawal_proposal_is_successfully {
    use super::*;

    #[test]
    pub fn initiated() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .insert_asset1_with_alice_public_key()
          .insert_a_valid_withdrawal_proposal()
          .set_threshold(2)
          .commit_a_valid_vote(true);

        let asset_balance_before_mint = Adapter::balance(
          context.valid_withdrawal.asset_id,
          &context.valid_withdrawal.account_id,
        );

        assert_ok!(Quorum::eval_proposal_state(
          context.alice,
          context.proposal_id
        ));

        assert!(Quorum::proposals()
          .iter()
          .find(|&&(proposal_id, _, _)| proposal_id == context.proposal_id)
          .is_some());

        assert_eq!(
          Quorum::proposal_votes(context.proposal_id).unwrap().status,
          ProposalStatus::Initiated
        );

        assert!(Quorum::proposal_votes(context.proposal_id)
          .unwrap()
          .votes_for
          .into_inner()
          .contains(&(ALICE_ACCOUNT_ID as u64)));

        assert_eq!(
          asset_balance_before_mint,
          Adapter::balance(
            context.valid_withdrawal.asset_id,
            &context.valid_withdrawal.account_id,
          )
        );
      });
    }

    #[test]
    pub fn rejected() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .insert_asset1_with_alice_public_key()
          .insert_a_valid_withdrawal_proposal()
          .commit_a_valid_vote(false);

        let asset_balance_before_withdrawal = Adapter::balance(
          context.valid_withdrawal.asset_id,
          &context.valid_withdrawal.account_id,
        );

        assert_ok!(Quorum::eval_proposal_state(
          context.alice,
          context.proposal_id
        ));

        assert!(Quorum::proposals()
          .iter()
          .find(|&&(proposal_id, _, _)| proposal_id == context.proposal_id)
          .is_none());

        assert!(Quorum::proposal_votes(context.proposal_id).is_none());

        assert_eq!(
          asset_balance_before_withdrawal,
          Adapter::balance(
            context.valid_withdrawal.asset_id,
            &context.valid_withdrawal.account_id,
          )
        );

        System::assert_has_event(MockEvent::Quorum(Event::ProposalRejected {
          proposal_id: context.proposal_id,
        }));
      });
    }
  }

  mod update_configuration_proposal_is_successfully {
    use super::*;

    #[test]
    pub fn initiated() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .insert_asset1_with_alice_public_key()
          .insert_a_valid_update_configuration_proposal()
          .set_threshold(2)
          .commit_a_valid_vote(true);

        assert_ok!(Quorum::eval_proposal_state(
          context.alice,
          context.proposal_id
        ));

        assert!(Quorum::proposals()
          .iter()
          .find(|&&(proposal_id, _, _)| proposal_id == context.proposal_id)
          .is_some());

        assert_eq!(
          Quorum::proposal_votes(context.proposal_id).unwrap().status,
          ProposalStatus::Initiated
        );

        assert!(Quorum::proposal_votes(context.proposal_id)
          .unwrap()
          .votes_for
          .into_inner()
          .contains(&(ALICE_ACCOUNT_ID as u64)));
      });
    }

    #[test]
    pub fn approved() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .insert_asset1_with_alice_public_key()
          .insert_a_valid_update_configuration_proposal()
          .commit_a_valid_vote(true);

        assert_ok!(Quorum::eval_proposal_state(
          context.alice,
          context.proposal_id
        ));

        assert!(Quorum::proposals()
          .iter()
          .find(|&&(proposal_id, _, _)| proposal_id == context.proposal_id)
          .is_none());

        assert!(Quorum::proposal_votes(context.proposal_id).is_none());

        System::assert_has_event(MockEvent::Quorum(Event::ProposalApproved {
          proposal_id: context.proposal_id,
        }));

        System::assert_has_event(MockEvent::Quorum(Event::ConfigurationUpdated {
          threshold: context.valid_update_configuration.threshold,
          members: context.valid_update_configuration.members.clone(),
        }));

        System::assert_has_event(MockEvent::Quorum(Event::ProposalProcessed {
          proposal_id: context.proposal_id,
        }));
      });
    }

    #[test]
    pub fn rejected() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .insert_asset1_with_alice_public_key()
          .insert_a_valid_update_configuration_proposal()
          .commit_a_valid_vote(false);

        assert_ok!(Quorum::eval_proposal_state(
          context.alice,
          context.proposal_id
        ));

        assert!(Quorum::proposals()
          .iter()
          .find(|&&(proposal_id, _, _)| proposal_id == context.proposal_id)
          .is_none());

        assert!(Quorum::proposal_votes(context.proposal_id).is_none());

        System::assert_has_event(MockEvent::Quorum(Event::ProposalRejected {
          proposal_id: context.proposal_id,
        }));
      });
    }
  }

  mod fails_when {
    use super::*;

    #[test]
    pub fn sender_is_not_signed() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .insert_asset1_with_alice_public_key()
          .insert_a_valid_mint_proposal_with_green_compliance_level()
          .commit_a_valid_vote(true);

        assert_noop!(
          Quorum::eval_proposal_state(Origin::none(), context.proposal_id),
          BadOrigin
        );
      });
    }

    #[test]
    pub fn proposal_is_already_completed() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .insert_asset1_with_alice_public_key()
          .insert_a_valid_mint_proposal_with_green_compliance_level()
          .set_valid_proposal_status(ProposalStatus::Approved);

        assert_noop!(
          Quorum::eval_proposal_state(context.alice, context.proposal_id),
          Error::<Test>::ProposalAlreadyComplete
        );
      });
    }

    #[test]
    pub fn proposal_is_expired() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .insert_asset1_with_alice_public_key()
          .insert_a_valid_mint_proposal_with_green_compliance_level()
          .commit_a_valid_vote(true);

        set_current_block(BLOCK_NUMBER_ZERO + ProposalLifetime::get() + 1);

        assert_noop!(
          Quorum::eval_proposal_state(context.alice, context.proposal_id),
          Error::<Test>::ProposalExpired
        );
      });
    }
  }
}

mod submit_public_keys {
  use super::*;

  #[test]
  pub fn succeeds() {
    new_test_ext().execute_with(|| {
      let context = Context::default().insert_asset1_with_alice_public_key();

      assert_ok!(Quorum::submit_public_keys(
        context.alice,
        vec![(TEMP_ASSET_ID, context.pub_key.clone())]
      ));

      assert!(Quorum::public_keys(ASSET_1).is_empty());

      let asset2_pub_key = Quorum::public_keys(TEMP_ASSET_ID).get(0).unwrap().clone();
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
          Quorum::submit_public_keys(Origin::none(), vec![(TEMP_ASSET_ID, context.pub_key)]),
          BadOrigin
        );
      });
    }

    #[test]
    pub fn sender_is_not_a_member() {
      new_test_ext().execute_with(|| {
        let context = Context::default().insert_asset1_with_alice_public_key();

        assert_noop!(
          Quorum::submit_public_keys(context.bob, vec![(TEMP_ASSET_ID, context.pub_key)]),
          Error::<Test>::AccessDenied
        );
      });
    }
  }
}
