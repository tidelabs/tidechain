use crate::{
  mock::{new_test_ext, Origin, Quorum, Test},
  pallet::*,
};
use frame_support::assert_ok;
use tidefi_primitives::{ComplianceLevel, CurrencyId, Hash, Mint, ProposalType};

#[test]
pub fn should_submit_proposal() {
  new_test_ext().execute_with(|| {
    assert!(Members::<Test>::contains_key(1));
  });
}

#[test]
pub fn should_vote_for_mint() {
  new_test_ext().execute_with(|| {
    let alice = Origin::signed(1u64);
    assert!(Members::<Test>::contains_key(1));
    let proposal = ProposalType::Mint(Mint {
      account_id: 1,
      currency_id: CurrencyId::Tide,
      mint_amount: 1_000_000_000_000,
      transaction_id: Vec::new(),
      compliance_level: ComplianceLevel::Green,
    });

    let proposal_id = Hash::zero();
    assert_ok!(Proposals::<Test>::try_append((proposal_id, proposal)));
    assert_ok!(Quorum::acknowledge_proposal(alice, proposal_id));
  });
}
