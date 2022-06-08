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
    new_test_ext, AccountId, Adapter, Balance, Origin, Security, System, Test, TidefiStaking,
    UnstakeQueueCap,
  },
  Error, UnstakeQueue,
};
use frame_support::{
  assert_noop, assert_ok,
  traits::{
    fungibles::{Inspect, Mutate},
    Hooks,
  },
  BoundedVec,
};
use sp_runtime::{traits::BadOrigin, Percent};
use std::str::FromStr;
use tidefi_primitives::{pallet::StakingExt, BlockNumber, CurrencyId, Hash};

const TEST_TOKEN: u32 = 2;
const TEST_TOKEN_CURRENCY_ID: CurrencyId = CurrencyId::Wrapped(TEST_TOKEN);
const FIFTEEN_DAYS: BlockNumber = 14400 * 15;
const BLOCKS_FORCE_UNLOCK: BlockNumber = 256;

// Asset Units
const ONE_TDFY: Balance = 1_000_000_000_000;
const ONE_TEST_TOKEN: Balance = 100;

// Test Accounts
const ALICE_ACCOUNT_ID: AccountId = 1;

const BLOCK_NUMBER_ZERO: BlockNumber = 0;

struct Context {
  staker: AccountId,
  tdfy_amount: Balance,
  test_token_amount: Balance,
  stake_id: Hash,
  duration: BlockNumber,
}

impl Default for Context {
  fn default() -> Self {
    Self {
      staker: ALICE_ACCOUNT_ID,
      tdfy_amount: ONE_TDFY,
      test_token_amount: ONE_TEST_TOKEN,
      stake_id: Hash::from_str(
        "0x02a204a25c36f8c88eea76e08cdaa22a0569ef630bf4416db72abb9fb2445f2b",
      )
      .unwrap_or_default(),
      duration: FIFTEEN_DAYS,
    }
  }
}

impl Context {
  fn mint_tdfy(self, account: AccountId, amount: Balance) -> Self {
    Self::mint_asset_for_accounts(vec![account], CurrencyId::Tdfy, amount);
    assert_eq!(Adapter::balance(CurrencyId::Tdfy, &account), amount);
    self
  }

  fn mint_test_token(self, account: AccountId, amount: Balance) -> Self {
    Self::mint_asset_for_accounts(vec![account], TEST_TOKEN_CURRENCY_ID, amount);
    assert_eq!(Adapter::balance(TEST_TOKEN_CURRENCY_ID, &account), amount);
    self
  }

  fn mint_asset_for_accounts(accounts: Vec<AccountId>, asset: CurrencyId, amount: u128) {
    for account in accounts {
      assert_ok!(Adapter::mint_into(asset, &account, amount));
    }
  }

  fn stake_tdfy(self) -> Self {
    assert_ok!(TidefiStaking::stake(
      Origin::signed(self.staker),
      CurrencyId::Tdfy,
      self.tdfy_amount,
      self.duration
    ));
    self
  }

  fn stake_test_tokens(self) -> Self {
    assert_ok!(TidefiStaking::stake(
      Origin::signed(self.staker),
      TEST_TOKEN_CURRENCY_ID,
      self.test_token_amount,
      self.duration
    ));
    self
  }

  fn set_current_block_to_pass_stake_duration(self, block_number: BlockNumber) -> Self {
    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = block_number;
      *n
    });
    self
  }

  fn add_mock_unstakes_to_queue(self, number_of_unstakes: u32) -> Self {
    UnstakeQueue::<Test>::put(
      BoundedVec::try_from(vec![
        (self.staker, self.stake_id, BLOCK_NUMBER_ZERO);
        number_of_unstakes as usize
      ])
      .unwrap(),
    );
    self
  }
}

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    assert_eq!(
      TidefiStaking::staking_rewards()
        .into_iter()
        .find(|(duration, _)| *duration == 14400 * 15),
      Some((14400 * 15, Percent::from_parts(2)))
    );
  });
}

mod stake {
  use super::*;

  mod succeeds {
    use super::*;

    #[test]
    fn for_native_asset() {
      new_test_ext().execute_with(|| {
        let context = Context::default().mint_tdfy(ALICE_ACCOUNT_ID, 1_000 * ONE_TDFY);

        assert_ok!(TidefiStaking::stake(
          Origin::signed(context.staker),
          CurrencyId::Tdfy,
          context.tdfy_amount,
          context.duration
        ));

        // make sure the staking pool has been updated
        assert_eq!(
          TidefiStaking::staking_pool(CurrencyId::Tdfy),
          Some(context.tdfy_amount)
        );

        // make sure the staking has been recorded in the storage
        assert!(TidefiStaking::account_stakes(context.staker).len() == 1);
        assert!(
          TidefiStaking::account_stakes(context.staker)
            .first()
            .unwrap()
            .initial_balance
            == context.tdfy_amount
        );
      });
    }

    #[test]
    fn for_wrapped_asset() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_test_token(ALICE_ACCOUNT_ID, 1_000 * ONE_TEST_TOKEN);

        assert_ok!(TidefiStaking::stake(
          Origin::signed(context.staker),
          TEST_TOKEN_CURRENCY_ID,
          context.test_token_amount,
          context.duration
        ));

        // make sure the staking pool has been updated
        assert_eq!(
          TidefiStaking::staking_pool(TEST_TOKEN_CURRENCY_ID),
          Some(context.test_token_amount)
        );

        // make sure the staking has been recorded in the storage
        assert!(TidefiStaking::account_stakes(context.staker).len() == 1);
        assert!(
          TidefiStaking::account_stakes(context.staker)
            .first()
            .unwrap()
            .initial_balance
            == context.test_token_amount
        );
      });
    }
  }

  mod fails_when {
    use super::*;

    #[test]
    fn not_signed() {
      new_test_ext().execute_with(|| {
        let context = Context::default().mint_tdfy(ALICE_ACCOUNT_ID, 1_000 * ONE_TDFY);

        assert_noop!(
          TidefiStaking::stake(
            Origin::none(),
            CurrencyId::Tdfy,
            context.tdfy_amount,
            context.duration
          ),
          BadOrigin
        );
      });
    }

    #[test]
    fn duration_is_invalid() {
      new_test_ext().execute_with(|| {
        let context = Context::default().mint_tdfy(ALICE_ACCOUNT_ID, 1_000 * ONE_TDFY);

        assert_noop!(
          TidefiStaking::stake(
            Origin::signed(context.staker),
            CurrencyId::Tdfy,
            context.tdfy_amount,
            1
          ),
          Error::<Test>::InvalidDuration
        );
      });
    }

    #[test]
    fn stake_amount_is_too_small() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_test_token(ALICE_ACCOUNT_ID, 1_000 * ONE_TEST_TOKEN);

        assert_noop!(
          TidefiStaking::stake(
            Origin::signed(context.staker),
            TEST_TOKEN_CURRENCY_ID,
            ONE_TEST_TOKEN - 1,
            context.duration
          ),
          Error::<Test>::AmountTooSmall
        );
      });
    }

    #[test]
    fn stake_amount_is_too_large() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_test_token(ALICE_ACCOUNT_ID, 1_000 * ONE_TEST_TOKEN);

        assert_noop!(
          TidefiStaking::stake(
            Origin::signed(context.staker),
            TEST_TOKEN_CURRENCY_ID,
            u128::MAX,
            context.duration
          ),
          Error::<Test>::AmountTooLarge
        );
      });
    }
  }
}

mod unstake {
  use super::*;

  mod succeeds {
    use super::*;

    mod with_force_unstake {
      use super::*;

      #[test]
      fn for_native_asset() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .mint_tdfy(ALICE_ACCOUNT_ID, 1_000 * ONE_TDFY)
            .stake_tdfy();

          assert_ok!(TidefiStaking::unstake(
            Origin::signed(context.staker),
            context.stake_id,
            true
          ));
        });
      }

      #[test]
      fn for_wrapped_asset() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .mint_tdfy(ALICE_ACCOUNT_ID, 1_000 * ONE_TDFY)
            .mint_test_token(ALICE_ACCOUNT_ID, 1_000 * ONE_TEST_TOKEN)
            .stake_test_tokens();

          assert_ok!(TidefiStaking::unstake(
            Origin::signed(context.staker),
            context.stake_id,
            true
          ));
        });
      }
    }
  }

  mod fails_when {
    use super::*;

    #[test]
    fn not_signed() {
      new_test_ext().execute_with(|| {
        Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 1_000 * ONE_TDFY)
          .stake_tdfy();

        assert_noop!(
          TidefiStaking::unstake(Origin::none(), Hash::zero(), true),
          BadOrigin
        );
      });
    }

    #[test]
    fn stake_id_is_invalid() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 1_000 * ONE_TDFY)
          .stake_tdfy();

        assert_noop!(
          TidefiStaking::unstake(Origin::signed(context.staker), Hash::zero(), true),
          Error::<Test>::InvalidStakeId
        );
      });
    }

    #[test]
    fn staking_is_not_ready() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 1_000 * ONE_TDFY)
          .stake_tdfy();

        assert_noop!(
          TidefiStaking::unstake(Origin::signed(context.staker), context.stake_id, false),
          Error::<Test>::StakingNotReady
        );
      });
    }

    #[test]
    fn native_asset_has_insufficient_balance() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .stake_tdfy();

        assert_noop!(
          TidefiStaking::unstake(Origin::signed(context.staker), context.stake_id, true),
          Error::<Test>::InsufficientBalance
        );
      });
    }

    #[test]
    fn wrapped_asset_has_insufficient_balance() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_test_token(ALICE_ACCOUNT_ID, ONE_TEST_TOKEN)
          .stake_test_tokens();

        assert_noop!(
          TidefiStaking::unstake(Origin::signed(context.staker), context.stake_id, true),
          Error::<Test>::InsufficientBalance
        );
      });
    }

    #[test]
    fn unstake_queue_exceeds_its_cap() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 2 * ONE_TDFY)
          .stake_tdfy()
          .add_mock_unstakes_to_queue(UnstakeQueueCap::get());

        assert_noop!(
          TidefiStaking::unstake(Origin::signed(context.staker), context.stake_id, true),
          Error::<Test>::UnstakeQueueCapExceeded
        );
      });
    }
  }
}

#[test]
pub fn should_stake_and_unstake() {
  new_test_ext().execute_with(|| {
    let alice = 1u64;
    let alice_origin = Origin::signed(alice);

    // mint token to user
    Adapter::mint_into(CurrencyId::Tdfy, &alice, 1_000_000_000_000_000)
      .expect("Unable to mint token");

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &1u64),
      1_000_000_000_000_000
    );

    assert_ok!(TidefiStaking::stake(
      alice_origin.clone(),
      CurrencyId::Tdfy,
      1_000_000_000_000,
      FIFTEEN_DAYS
    ));

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &1u64),
      1_000_000_000_000_000 - 1_000_000_000_000
    );

    let stake_id = TidefiStaking::account_stakes(alice)
      .first()
      .unwrap()
      .unique_id;

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tdfy),
      Some(1_000_000_000_000)
    );

    // make sure the staking has been recorded in the storage
    assert!(TidefiStaking::account_stakes(alice).len() == 1);
    assert!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .initial_balance
        == 1_000_000_000_000
    );

    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS + 1;
      *n
    });

    assert_eq!(Security::current_block_number(), FIFTEEN_DAYS + 1);
    assert_ok!(TidefiStaking::unstake(alice_origin, stake_id, false));
    assert!(TidefiStaking::account_stakes(alice).len() == 0);
    // balance is returned
    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &1u64),
      1_000_000_000_000_000
    );
  });
}

#[test]
pub fn should_stake_and_unstake_queue() {
  new_test_ext().execute_with(|| {
    let alice = 1u64;
    let alice_origin = Origin::signed(alice);
    let initial_stake = 1_000_000_000_000;
    let initial_mint = 1_000_000_000_000_000;

    // mint token to user
    Adapter::mint_into(CurrencyId::Tdfy, &alice, initial_mint).expect("Unable to mint token");

    assert_eq!(Adapter::balance(CurrencyId::Tdfy, &1u64), initial_mint);

    assert_ok!(TidefiStaking::stake(
      alice_origin.clone(),
      CurrencyId::Tdfy,
      initial_stake,
      FIFTEEN_DAYS
    ));

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &1u64),
      initial_mint - initial_stake
    );

    let stake_id = TidefiStaking::account_stakes(alice)
      .first()
      .unwrap()
      .unique_id;

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tdfy),
      Some(initial_stake)
    );

    // make sure the staking has been recorded in the storage
    assert!(TidefiStaking::account_stakes(alice).len() == 1);
    assert!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .initial_balance
        == initial_stake
    );

    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS - 1_000;
      *n
    });

    assert_eq!(Security::current_block_number(), FIFTEEN_DAYS - 1_000);
    assert_ok!(TidefiStaking::unstake(alice_origin, stake_id, true));
    assert!(TidefiStaking::account_stakes(alice).len() == 1);

    let unstake_fee = TidefiStaking::unstake_fee() * initial_stake;
    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &1u64),
      initial_mint - initial_stake - unstake_fee
    );
    // 1 % of 1_000_000_000_000 = 10_000_000_000
    assert_eq!(unstake_fee, 10_000_000_000);

    // BlocksForceUnstake is set to 10, so let skip at least 10 blocks
    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS - 1_000 + BLOCKS_FORCE_UNLOCK + 1;
      *n
    });
    assert_eq!(
      Security::current_block_number(),
      FIFTEEN_DAYS - 1_000 + BLOCKS_FORCE_UNLOCK + 1
    );

    assert!(TidefiStaking::unstake_queue().len() == 1);

    // run on idle hook
    assert_eq!(
      TidefiStaking::on_idle(1, 1_000_000_000_000_000),
      1_000_000_000_000_000
    );

    assert!(TidefiStaking::unstake_queue().len() == 0);
  });
}

#[test]
pub fn should_stake_multiple_and_unstake_queue() {
  new_test_ext().execute_with(|| {
    let alice = 1u64;
    let alice_origin = Origin::signed(alice);

    let bob = 2u64;
    let bob_origin = Origin::signed(bob);

    let initial_stake = 1_000_000_000_000;
    let initial_stake_bob = initial_stake / 4;
    let initial_mint = 1_000_000_000_000_000;

    // mint token to user
    Adapter::mint_into(CurrencyId::Tdfy, &alice, initial_mint).expect("Unable to mint token");
    Adapter::mint_into(CurrencyId::Tdfy, &bob, initial_mint).expect("Unable to mint token");

    assert_eq!(Adapter::balance(CurrencyId::Tdfy, &alice), initial_mint);
    assert_eq!(Adapter::balance(CurrencyId::Tdfy, &bob), initial_mint);

    assert_ok!(TidefiStaking::stake(
      alice_origin.clone(),
      CurrencyId::Tdfy,
      initial_stake,
      FIFTEEN_DAYS
    ));

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &1u64),
      initial_mint - initial_stake
    );

    let stake_id = TidefiStaking::account_stakes(alice)
      .first()
      .unwrap()
      .unique_id;

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tdfy),
      Some(initial_stake)
    );

    // make sure the staking has been recorded in the storage
    assert!(TidefiStaking::account_stakes(alice).len() == 1);
    assert!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .initial_balance
        == initial_stake
    );

    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS - 3_000;
      *n
    });
    assert_eq!(Security::current_block_number(), FIFTEEN_DAYS - 3_000);

    assert_ok!(TidefiStaking::stake(
      bob_origin.clone(),
      CurrencyId::Tdfy,
      initial_stake_bob,
      FIFTEEN_DAYS
    ));

    let bob_stake_id = TidefiStaking::account_stakes(bob)
      .first()
      .unwrap()
      .unique_id;

    assert_ok!(TidefiStaking::stake(
      bob_origin.clone(),
      CurrencyId::Tdfy,
      initial_stake_bob,
      FIFTEEN_DAYS * 2
    ));

    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS - 2_000;
      *n
    });
    assert_eq!(Security::current_block_number(), FIFTEEN_DAYS - 2_000);

    assert_ok!(TidefiStaking::unstake(bob_origin, bob_stake_id, true));

    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS - 2_000 + (BLOCKS_FORCE_UNLOCK / 2);
      *n
    });
    assert_eq!(
      Security::current_block_number(),
      FIFTEEN_DAYS - 2_000 + (BLOCKS_FORCE_UNLOCK / 2)
    );
    assert_eq!(TidefiStaking::unstake_queue().len(), 1);
    assert_ok!(TidefiStaking::unstake(alice_origin, stake_id, true));
    assert_eq!(TidefiStaking::unstake_queue().len(), 2);

    assert_eq!(TidefiStaking::account_stakes(alice).len(), 1);
    assert_eq!(TidefiStaking::account_stakes(bob).len(), 2);

    let unstake_fee = TidefiStaking::unstake_fee() * initial_stake;
    let unstake_fee_bob = TidefiStaking::unstake_fee() * initial_stake_bob;

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &1u64),
      initial_mint - initial_stake - unstake_fee
    );

    // 1 % of 1_000_000_000_000 = 10_000_000_000
    assert_eq!(unstake_fee, 10_000_000_000);

    // BlocksForceUnstake is set to 10, so let skip at least 10 blocks
    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS - 2_000 + BLOCKS_FORCE_UNLOCK + 1;
      *n
    });

    assert_eq!(
      Security::current_block_number(),
      FIFTEEN_DAYS - 2_000 + BLOCKS_FORCE_UNLOCK + 1
    );

    assert_eq!(TidefiStaking::unstake_queue().len(), 2);

    // run on idle hook
    assert_eq!(
      TidefiStaking::on_idle(1, 1_000_000_000_000_000),
      1_000_000_000_000_000
    );

    assert_eq!(TidefiStaking::unstake_queue().len(), 1);

    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS - 2_000 + BLOCKS_FORCE_UNLOCK + (BLOCKS_FORCE_UNLOCK / 2) + 1;
      *n
    });

    assert_eq!(
      Security::current_block_number(),
      FIFTEEN_DAYS - 2_000 + BLOCKS_FORCE_UNLOCK + (BLOCKS_FORCE_UNLOCK / 2) + 1
    );
    // run on idle hook
    assert_eq!(
      TidefiStaking::on_idle(1, 1_000_000_000_000_000),
      1_000_000_000_000_000
    );

    assert!(TidefiStaking::unstake_queue().is_empty());

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &1u64),
      initial_mint - unstake_fee
    );

    assert!(TidefiStaking::account_stakes(bob).len() == 1);
    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &2u64),
      // we still have a stake active
      initial_mint - unstake_fee_bob - initial_stake_bob
    );
  });
}

#[test]
pub fn should_calculate_rewards() {
  new_test_ext().execute_with(|| {
    let alice = 1u64;
    let alice_origin = Origin::signed(alice);
    let bob = 2u64;
    let bob_origin = Origin::signed(bob);
    let charlie = 3u64;
    let charlie_origin = Origin::signed(charlie);

    System::set_block_number(1);

    // mint token to user
    Adapter::mint_into(CurrencyId::Tdfy, &alice, 1_000_000_000_000_000)
      .expect("Unable to mint token");
    Adapter::mint_into(CurrencyId::Tdfy, &bob, 1_000_000_000_000_000)
      .expect("Unable to mint token");
    Adapter::mint_into(CurrencyId::Tdfy, &charlie, 1_000_000_000_000_000)
      .expect("Unable to mint token");

    assert_ok!(TidefiStaking::stake(
      alice_origin,
      CurrencyId::Tdfy,
      100_000_000_000_000,
      FIFTEEN_DAYS
    ));

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tdfy),
      Some(100_000_000_000_000)
    );

    // make sure the staking has been recorded in the storage
    assert!(TidefiStaking::account_stakes(alice).len() == 1);
    assert_eq!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .initial_balance,
      100_000_000_000_000
    );

    // 100 for TDFY in fees for session 1
    // 15 days should get 2%, so 2 tides
    assert_ok!(TidefiStaking::on_session_end(
      1,
      vec![(CurrencyId::Tdfy, 100_000_000_000_000)]
    ));

    // run on idle hook
    assert_eq!(
      TidefiStaking::on_idle(1, 1_000_000_000_000_000),
      1_000_000_000_000_000
    );

    // started with 100, now should have 102 tides
    assert_eq!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .principal,
      102_000_000_000_000
    );

    assert_ok!(TidefiStaking::stake(
      bob_origin,
      CurrencyId::Tdfy,
      100_000_000_000_000,
      FIFTEEN_DAYS
    ));

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tdfy),
      Some(200_000_000_000_000)
    );

    // 100 for TDFY in fees for session 1
    // 15 days should get 2%, so 2 tides
    assert_ok!(TidefiStaking::on_session_end(
      2,
      vec![(CurrencyId::Tdfy, 100_000_000_000_000)]
    ));

    // run on idle hook
    assert_eq!(
      TidefiStaking::on_idle(1, 1_000_000_000_000_000),
      1_000_000_000_000_000
    );

    assert_eq!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .principal,
      103_000_000_000_000
    );

    assert_eq!(
      TidefiStaking::account_stakes(bob)
        .first()
        .unwrap()
        .principal,
      101_000_000_000_000
    );

    // 2 empty sessions
    assert_ok!(TidefiStaking::on_session_end(3, Vec::new()));
    assert_ok!(TidefiStaking::on_session_end(4, Vec::new()));
    assert_eq!(
      TidefiStaking::on_idle(1, 1_000_000_000_000_000),
      1_000_000_000_000_000
    );

    assert_eq!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .principal,
      103_000_000_000_000
    );

    assert_eq!(
      TidefiStaking::account_stakes(bob)
        .first()
        .unwrap()
        .principal,
      101_000_000_000_000
    );

    assert_ok!(TidefiStaking::stake(
      charlie_origin,
      CurrencyId::Tdfy,
      400_000_000_000_000,
      FIFTEEN_DAYS
    ));

    assert_ok!(TidefiStaking::on_session_end(
      5,
      vec![(CurrencyId::Tdfy, 100_000_000_000_000)]
    ));

    assert_eq!(
      TidefiStaking::on_idle(1, 1_000_000_000_000_000),
      1_000_000_000_000_000
    );

    assert_eq!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .principal,
      103_333_333_333_333
    );

    assert_eq!(
      TidefiStaking::account_stakes(bob)
        .first()
        .unwrap()
        .principal,
      101_333_333_333_333
    );

    assert_eq!(
      TidefiStaking::account_stakes(charlie)
        .first()
        .unwrap()
        .principal,
      401_333_333_333_333
    );
  });
}
