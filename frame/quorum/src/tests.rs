use crate::{
  mock::{new_test_ext, Origin, ProposalLifetime, Quorum, Security, Test},
  pallet::*,
};
use frame_support::{assert_ok, traits::Hooks};
use tidefi_primitives::{
  pallet::SecurityExt, ComplianceLevel, CurrencyId, Hash, Mint, ProposalType,
};

#[test]
pub fn should_submit_proposal() {
  new_test_ext().execute_with(|| {
    let alice = Origin::signed(1u64);
    PublicKeys::<Test>::insert(1, 1, "pubkey".as_bytes());
    assert!(Members::<Test>::contains_key(1));
    assert_eq!(PublicKeys::<Test>::get(1, 1), "pubkey".as_bytes());

    let proposal = ProposalType::Mint(Mint {
      account_id: 1,
      currency_id: CurrencyId::Tide,
      mint_amount: 1_000_000_000_000,
      transaction_id: Vec::new(),
      compliance_level: ComplianceLevel::Green,
    });
    assert_ok!(Quorum::submit_proposal(alice, proposal));
  });
}

#[test]
pub fn should_vote_for_mint() {
  new_test_ext().execute_with(|| {
    let alice = Origin::signed(1u64);
    PublicKeys::<Test>::insert(1, 1, "pubkey".as_bytes());
    assert!(Members::<Test>::contains_key(1));
    assert_eq!(PublicKeys::<Test>::get(1, 1), "pubkey".as_bytes());
    let proposal = ProposalType::Mint(Mint {
      account_id: 1,
      currency_id: CurrencyId::Tide,
      mint_amount: 1_000_000_000_000,
      transaction_id: Vec::new(),
      compliance_level: ComplianceLevel::Green,
    });

    let proposal_id = Hash::zero();
    assert_ok!(Proposals::<Test>::try_append((
      proposal_id,
      Security::get_current_block_count(),
      proposal
    )));
    assert_ok!(Quorum::acknowledge_proposal(alice, proposal_id));
  });
}

#[test]
pub fn should_remove_expired() {
  new_test_ext().execute_with(|| {
    let alice = Origin::signed(1u64);
    PublicKeys::<Test>::insert(1, 1, "pubkey".as_bytes());
    assert!(Members::<Test>::contains_key(1));
    assert_eq!(PublicKeys::<Test>::get(1, 1), "pubkey".as_bytes());
    let proposal = ProposalType::Mint(Mint {
      account_id: 1,
      currency_id: CurrencyId::Tide,
      mint_amount: 1_000_000_000_000,
      transaction_id: Vec::new(),
      compliance_level: ComplianceLevel::Green,
    });
    assert_ok!(Quorum::submit_proposal(alice, proposal));
    assert_eq!(Quorum::on_idle(0, 1_000_000_000_000), 0);
    assert_eq!(Proposals::<Test>::get().len(), 1);

    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = ProposalLifetime::get() + 2;
      *n
    });
    assert_eq!(Quorum::on_idle(0, 1_000_000_000_000), 0);
    assert_eq!(Proposals::<Test>::get().len(), 0);
  });
}
