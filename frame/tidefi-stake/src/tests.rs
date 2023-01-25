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
    new_test_ext, AccountId, Adapter, Balance, FeeAmount, MarketMakerFeeAmount, Oracle, Origin,
    Security, StakeAccountCap, Test, TidefiStaking, UnstakeQueueCap,
  },
  pallet as pallet_tidefi_stake, AccountStakes, Error, StakingPool, UnstakeQueue,
};
use frame_support::{
  assert_noop, assert_ok,
  traits::{
    fungibles::{Inspect, InspectHold, Mutate},
    Hooks,
  },
  BoundedVec,
};
use sp_runtime::{
  traits::{AccountIdConversion, BadOrigin},
  ArithmeticError, DispatchError, Percent, Permill,
};
use std::str::FromStr;
use tidefi_primitives::{
  pallet::{OracleExt, StakingExt},
  BlockNumber, CurrencyId, Hash, Stake, StakeStatus, SwapConfirmation, SwapStatus, SwapType,
};

const TEST_TOKEN: u32 = 2;
const TEST_TOKEN_CURRENCY_ID: CurrencyId = CurrencyId::Wrapped(TEST_TOKEN);
const FIFTEEN_DAYS: BlockNumber = 14400 * 15;
const BLOCKS_FORCE_UNLOCK: BlockNumber = 256;

// Extrinsic Hashes
const EXTRINSIC_HASH_0: [u8; 32] = [0; 32];
const EXTRINSIC_HASH_1: [u8; 32] = [1; 32];

// Slippage Rates
const SLIPPAGE_0_PERCENT: Permill = Permill::from_percent(0);

// Swap Fee Rates
const REQUESTER_SWAP_FEE_RATE: Permill = FeeAmount::get();
const MARKET_MAKER_SWAP_FEE_RATE: Permill = MarketMakerFeeAmount::get();

// Asset Units
const ONE_TDFY: Balance = 1_000_000_000_000;
const ONE_TEST_TOKEN: Balance = 100_000_000;

// Test Accounts
const ALICE_ACCOUNT_ID: AccountId = 1;
const BOB_ACCOUNT_ID: AccountId = 2;
const CHARLIE_ACCOUNT_ID: AccountId = 3;

const ALICE_INITIAL_ONE_THOUSAND_TDFYS: Balance = 1_000 * ONE_TDFY;
const ALICE_INITIAL_ONE_THOUSAND_TEST_TOKENS: Balance = 1_000 * ONE_TEST_TOKEN;
const BOB_INITIAL_ONE_THOUSAND_TDFYS: Balance = 1_000 * ONE_TDFY;
const BOB_INITIAL_ONE_THOUSAND_TEST_TOKENS: Balance = 1_000 * ONE_TEST_TOKEN;
const CHARLIE_INITIAL_ONE_THOUSAND_TDFYS: Balance = 1_000 * ONE_TDFY;
const CHARLIE_INITIAL_ONE_THOUSAND_TEST_TOKENS: Balance = 1_000 * ONE_TEST_TOKEN;

const ALICE_STAKE_ONE_TDFY: Balance = ONE_TDFY;
const BOB_STAKE_QUARTER_TDFY: Balance = ALICE_STAKE_ONE_TDFY / 4;

const BLOCK_NUMBER_ZERO: BlockNumber = 0;

struct Context {
  staker: AccountId,
  staking_pallet_account: AccountId,
  tdfy_amount: Balance,
  test_token_amount: Balance,
  stake_id: Hash,
  duration: BlockNumber,
  market_makers: Vec<AccountId>,
}

impl Default for Context {
  fn default() -> Self {
    Self {
      staker: ALICE_ACCOUNT_ID,
      staking_pallet_account: <Test as pallet_tidefi_stake::Config>::StakePalletId::get()
        .into_account_truncating(),
      tdfy_amount: ONE_TDFY,
      test_token_amount: ONE_TEST_TOKEN,
      stake_id: Hash::from_str(
        "0x02a204a25c36f8c88eea76e08cdaa22a0569ef630bf4416db72abb9fb2445f2b",
      )
      .unwrap_or_default(),
      duration: FIFTEEN_DAYS,
      market_makers: vec![],
    }
  }
}

impl Context {
  fn set_oracle_status(self, status: bool) -> Self {
    assert_ok!(Oracle::set_status(Origin::signed(ALICE_ACCOUNT_ID), status));
    match status {
      true => assert!(Oracle::status()),
      false => assert!(!Oracle::status()),
    }
    self
  }

  fn set_market_makers(mut self, account_ids: Vec<AccountId>) -> Self {
    self.market_makers = account_ids;
    self
  }

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
    assert_eq!(
      0,
      Adapter::balance(CurrencyId::Tdfy, &self.staking_pallet_account)
    );

    assert_ok!(TidefiStaking::stake(
      Origin::signed(self.staker),
      CurrencyId::Tdfy,
      self.tdfy_amount,
      self.duration
    ));

    assert_eq!(
      self.tdfy_amount,
      Adapter::balance(CurrencyId::Tdfy, &self.staking_pallet_account)
    );

    self
  }

  fn stake_test_tokens(self) -> Self {
    assert_eq!(
      0,
      Adapter::balance(TEST_TOKEN_CURRENCY_ID, &self.staking_pallet_account)
    );

    assert_ok!(TidefiStaking::stake(
      Origin::signed(self.staker),
      TEST_TOKEN_CURRENCY_ID,
      self.test_token_amount,
      self.duration
    ));

    assert_eq!(
      self.test_token_amount,
      Adapter::balance(TEST_TOKEN_CURRENCY_ID, &self.staking_pallet_account)
    );

    self
  }

  fn add_mock_unstakes_to_queue(self, number_of_unstakes: usize) -> Self {
    UnstakeQueue::<Test>::put(
      BoundedVec::try_from(vec![
        (self.staker, self.stake_id, BLOCK_NUMBER_ZERO);
        number_of_unstakes as usize
      ])
      .unwrap(),
    );
    self
  }

  fn add_mock_account_stakes(self, account_id: AccountId, number_of_stakes: usize) -> Self {
    AccountStakes::<Test>::insert(
      &account_id,
      BoundedVec::try_from(vec![
        Stake {
          currency_id: CurrencyId::Tdfy,
          unique_id: Hash::zero(),
          last_session_index_compound: 0,
          initial_block: BLOCK_NUMBER_ZERO,
          initial_balance: self.tdfy_amount,
          principal: self.tdfy_amount,
          duration: self.duration,
          status: Default::default(),
        };
        number_of_stakes
      ])
      .unwrap(),
    );
    self
  }

  fn insert_asset_balance_in_staking_pool_to_max(
    self,
    currency_id: CurrencyId,
    new_balance: Balance,
  ) -> Self {
    StakingPool::<Test>::insert(currency_id, new_balance);
    self
  }

  fn create_tdfy_to_temp_limit_swap_request(
    &self,
    requester_account_id: AccountId,
    tdfy_amount: Balance,
    temp_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      CurrencyId::Tdfy,
      tdfy_amount,
      TEST_TOKEN_CURRENCY_ID,
      temp_amount,
      BLOCK_NUMBER_ZERO,
      extrinsic_hash,
      self.market_makers.contains(&requester_account_id),
      SwapType::Limit,
      slippage,
    )
  }

  fn create_temp_to_tdfy_limit_swap_request(
    &self,
    requester_account_id: AccountId,
    temp_amount: Balance,
    tdfy_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      TEST_TOKEN_CURRENCY_ID,
      temp_amount,
      CurrencyId::Tdfy,
      tdfy_amount,
      BLOCK_NUMBER_ZERO,
      extrinsic_hash,
      self.market_makers.contains(&requester_account_id),
      SwapType::Limit,
      slippage,
    )
  }
}

fn add_new_swap_and_assert_results(
  account_id: AccountId,
  asset_id_from: CurrencyId,
  amount_from: Balance,
  asset_id_to: CurrencyId,
  amount_to: Balance,
  block_number: BlockNumber,
  extrinsic_hash: [u8; 32],
  is_market_maker: bool,
  swap_type: SwapType,
  slippage: Permill,
) -> Hash {
  let initial_from_token_balance = Adapter::balance(asset_id_from, &account_id);

  let (trade_request_id, trade_request) = Oracle::add_new_swap_in_queue(
    account_id,
    asset_id_from,
    amount_from,
    asset_id_to,
    amount_to,
    block_number,
    extrinsic_hash,
    is_market_maker,
    swap_type,
    slippage,
  )
  .unwrap();

  assert_swap_cost_is_suspended(account_id, asset_id_from, amount_from, is_market_maker);

  if asset_id_from != CurrencyId::Tdfy {
    assert_sold_tokens_are_deducted(
      account_id,
      asset_id_from,
      initial_from_token_balance,
      amount_from,
      is_market_maker,
    );
  }

  assert_spendable_balance_is_updated(
    account_id,
    asset_id_from,
    initial_from_token_balance,
    amount_from,
    is_market_maker,
  );

  assert_eq!(trade_request.status, SwapStatus::Pending);
  assert_eq!(
    Oracle::account_swaps(account_id)
      .unwrap()
      .iter()
      .find(|(request_id, _)| *request_id == trade_request_id),
    Some(&(trade_request_id, SwapStatus::Pending))
  );

  assert_eq!(trade_request.block_number, BLOCK_NUMBER_ZERO);

  trade_request_id
}

fn assert_swap_cost_is_suspended(
  account_id: AccountId,
  currency_id: CurrencyId,
  sell_amount: Balance,
  is_market_maker: bool,
) {
  let swap_fee_rate = if is_market_maker {
    MARKET_MAKER_SWAP_FEE_RATE
  } else {
    REQUESTER_SWAP_FEE_RATE
  };

  assert_eq!(
    Adapter::balance_on_hold(currency_id, &account_id),
    sell_amount.saturating_add(swap_fee_rate * sell_amount)
  );
}

fn assert_spendable_balance_is_updated(
  account_id: AccountId,
  currency_id: CurrencyId,
  initial_balance: Balance,
  sell_amount: Balance,
  is_market_maker: bool,
) {
  let swap_fee_rate = if is_market_maker {
    MARKET_MAKER_SWAP_FEE_RATE
  } else {
    REQUESTER_SWAP_FEE_RATE
  };

  let expected_reducible_balance = initial_balance
    .saturating_sub(sell_amount)
    .saturating_sub(swap_fee_rate * sell_amount);

  match currency_id {
    CurrencyId::Tdfy => assert_eq!(
      Adapter::reducible_balance(currency_id, &account_id, true),
      expected_reducible_balance
    ),
    _ => assert_eq!(
      Adapter::reducible_balance(currency_id, &account_id, true),
      expected_reducible_balance.saturating_sub(1_u128) // keep-alive token
    ),
  }

  assert_eq!(
    Adapter::reducible_balance(currency_id, &account_id, false),
    initial_balance
      .saturating_sub(sell_amount)
      .saturating_sub(swap_fee_rate * sell_amount)
  );
}

fn assert_sold_tokens_are_deducted(
  account_id: AccountId,
  currency_id: CurrencyId,
  initial_balance: Balance,
  sell_amount: Balance,
  is_market_maker: bool,
) {
  let swap_fee_rate = if is_market_maker {
    MARKET_MAKER_SWAP_FEE_RATE
  } else {
    REQUESTER_SWAP_FEE_RATE
  };

  assert_eq!(
    Adapter::balance(currency_id, &account_id),
    initial_balance
      .saturating_sub(sell_amount)
      .saturating_sub(swap_fee_rate * sell_amount)
  );
}

fn set_current_block(block_number: BlockNumber) {
  <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
    *n = block_number;
    *n
  });
}

fn run_on_idle_hook(block_number: BlockNumber, remaining_weights: Balance) {
  let weights: u64 = remaining_weights.try_into().unwrap();
  assert_eq!(TidefiStaking::on_idle(block_number, weights), weights);
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
        let context =
          Context::default().mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS);

        let staker_balance_before = Adapter::balance(CurrencyId::Tdfy, &context.staker);

        assert_ok!(TidefiStaking::stake(
          Origin::signed(context.staker),
          CurrencyId::Tdfy,
          context.tdfy_amount,
          context.duration
        ));

        assert_eq!(
          staker_balance_before - context.tdfy_amount,
          Adapter::balance(CurrencyId::Tdfy, &context.staker)
        );

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
          .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
          .mint_test_token(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TEST_TOKENS);

        let staker_balance_before = Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.staker);

        assert_ok!(TidefiStaking::stake(
          Origin::signed(context.staker),
          TEST_TOKEN_CURRENCY_ID,
          context.test_token_amount,
          context.duration
        ));

        assert_eq!(
          staker_balance_before - context.test_token_amount,
          Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.staker)
        );

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
        let context =
          Context::default().mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS);

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
        let context =
          Context::default().mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS);

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
          .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
          .mint_test_token(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TEST_TOKENS);

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
          .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
          .mint_test_token(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TEST_TOKENS);

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

    #[ignore]
    #[test]
    fn staking_pool_reaches_its_cap() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
          .mint_test_token(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TEST_TOKENS)
          .insert_asset_balance_in_staking_pool_to_max(TEST_TOKEN_CURRENCY_ID, u128::MAX);

        assert_noop!(
          TidefiStaking::stake(
            Origin::signed(context.staker),
            TEST_TOKEN_CURRENCY_ID,
            context.test_token_amount,
            context.duration
          ),
          ArithmeticError::Overflow
        );
      });
    }

    #[ignore]
    #[test]
    fn account_stakes_reaches_its_cap() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 2 * ONE_TDFY)
          .add_mock_account_stakes(ALICE_ACCOUNT_ID, StakeAccountCap::get() as usize);

        assert_noop!(
          TidefiStaking::stake(
            Origin::signed(context.staker),
            CurrencyId::Tdfy,
            context.tdfy_amount,
            context.duration
          ),
          DispatchError::Other("Invalid stake; eqd")
        );
      });
    }
  }
}

mod unstake {
  use super::*;

  mod succeeds {
    use super::*;

    mod after_unstaking_becomes_ready {
      use super::*;

      #[test]
      fn for_native_asset() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
            .stake_tdfy();

          set_current_block(FIFTEEN_DAYS + 1);

          let staker_balance_before = Adapter::balance(CurrencyId::Tdfy, &context.staker);

          assert_ok!(TidefiStaking::unstake(
            Origin::signed(context.staker),
            context.stake_id,
            false
          ));

          assert_eq!(
            staker_balance_before + context.tdfy_amount,
            Adapter::balance(CurrencyId::Tdfy, &context.staker)
          );

          // Staking pallet account becomes empty
          assert_eq!(
            0,
            Adapter::balance(CurrencyId::Tdfy, &context.staking_pallet_account)
          );
        });
      }

      #[test]
      fn for_wrapped_asset() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .set_oracle_status(true)
            .set_market_makers(vec![ALICE_ACCOUNT_ID])
            .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
            .mint_test_token(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TEST_TOKENS)
            .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_ONE_THOUSAND_TDFYS)
            .mint_test_token(BOB_ACCOUNT_ID, BOB_INITIAL_ONE_THOUSAND_TEST_TOKENS)
            .mint_tdfy(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_ONE_THOUSAND_TDFYS)
            .mint_test_token(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_ONE_THOUSAND_TEST_TOKENS)
            .stake_test_tokens();

          // Swaps
          let trade_request_id_1 = context.create_tdfy_to_temp_limit_swap_request(
            BOB_ACCOUNT_ID,
            10 * ONE_TDFY,
            (2 * ONE_TEST_TOKEN).into(),
            EXTRINSIC_HASH_0,
            SLIPPAGE_0_PERCENT,
          );

          let trade_request_id_2 = context.create_temp_to_tdfy_limit_swap_request(
            CHARLIE_ACCOUNT_ID,
            (2 * ONE_TEST_TOKEN).into(),
            10 * ONE_TDFY,
            EXTRINSIC_HASH_1,
            SLIPPAGE_0_PERCENT,
          );

          // Filling
          assert_ok!(Oracle::confirm_swap(
            Origin::signed(ALICE_ACCOUNT_ID),
            trade_request_id_1,
            vec![SwapConfirmation {
              request_id: trade_request_id_2,
              amount_to_receive: (2 * ONE_TEST_TOKEN).into(),
              amount_to_send: 10 * ONE_TDFY
            },],
          ));

          assert!(Oracle::swaps(trade_request_id_1).is_none());
          assert!(Oracle::swaps(trade_request_id_2).is_some());

          set_current_block(FIFTEEN_DAYS + 1);

          let staker_balance_before = Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.staker);

          // TODO: Assert Swap event

          assert_ok!(TidefiStaking::unstake(
            Origin::signed(context.staker),
            context.stake_id,
            false
          ));

          // TODO: Assert Unstaked event

          // TODO: Update balance amount including the swap fee earned
          assert_eq!(
            staker_balance_before + context.test_token_amount,
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.staker)
          );

          // Staking pallet account becomes empty
          assert_eq!(
            0,
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.staking_pallet_account)
          );
        });
      }
    }

    mod when_force_unstake_before_staking_duration_expires {
      use super::*;

      #[test]
      fn for_native_asset() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
            .stake_tdfy();

          let staker_balance_before = Adapter::balance(CurrencyId::Tdfy, &context.staker);

          assert_ok!(TidefiStaking::unstake(
            Origin::signed(context.staker),
            context.stake_id,
            true
          ));

          assert_eq!(
            AccountStakes::<Test>::get(context.staker)
              .first()
              .unwrap()
              .status,
            StakeStatus::PendingUnlock(256)
          );

          let unstaking_fee = TidefiStaking::unstake_fee() * context.tdfy_amount;
          assert_eq!(
            staker_balance_before - unstaking_fee,
            Adapter::balance(CurrencyId::Tdfy, &context.staker)
          );
        });
      }

      #[test]
      fn for_wrapped_asset() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
            .mint_test_token(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TEST_TOKENS)
            .stake_test_tokens();

          let staker_balance_before = Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.staker);

          assert_ok!(TidefiStaking::unstake(
            Origin::signed(context.staker),
            context.stake_id,
            true
          ));

          assert_eq!(
            AccountStakes::<Test>::get(context.staker)
              .first()
              .unwrap()
              .status,
            StakeStatus::PendingUnlock(256)
          );

          let unstaking_fee = TidefiStaking::unstake_fee() * context.test_token_amount;
          assert_eq!(
            staker_balance_before - unstaking_fee,
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.staker)
          );
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
          .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
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
          .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
          .stake_tdfy();

        assert_noop!(
          TidefiStaking::unstake(Origin::signed(context.staker), Hash::zero(), true),
          Error::<Test>::InvalidStakeId
        );
      });
    }

    #[test]
    fn staking_is_not_expired_yet_and_not_force_unstake() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
          .stake_tdfy();

        assert_noop!(
          TidefiStaking::unstake(Origin::signed(context.staker), context.stake_id, false),
          Error::<Test>::UnstakingNotReady
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
          .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
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
          .add_mock_unstakes_to_queue(UnstakeQueueCap::get() as usize);

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
    Context::default().mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS);

    assert_ok!(TidefiStaking::stake(
      Origin::signed(ALICE_ACCOUNT_ID),
      CurrencyId::Tdfy,
      ALICE_STAKE_ONE_TDFY,
      FIFTEEN_DAYS
    ));

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &ALICE_ACCOUNT_ID),
      ALICE_INITIAL_ONE_THOUSAND_TDFYS - ALICE_STAKE_ONE_TDFY
    );

    let stake_id = TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
      .first()
      .unwrap()
      .unique_id;

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tdfy),
      Some(ALICE_STAKE_ONE_TDFY)
    );

    // make sure the staking has been recorded in the storage
    assert!(TidefiStaking::account_stakes(ALICE_ACCOUNT_ID).len() == 1);
    assert!(
      TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
        .first()
        .unwrap()
        .initial_balance
        == ALICE_STAKE_ONE_TDFY
    );

    set_current_block(FIFTEEN_DAYS + 1);

    assert_eq!(Security::current_block_number(), FIFTEEN_DAYS + 1);
    assert_ok!(TidefiStaking::unstake(
      Origin::signed(ALICE_ACCOUNT_ID),
      stake_id,
      false
    ));
    assert!(TidefiStaking::account_stakes(ALICE_ACCOUNT_ID).len() == 0);
    // balance is returned
    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &ALICE_ACCOUNT_ID),
      ALICE_INITIAL_ONE_THOUSAND_TDFYS
    );
  });
}

#[test]
pub fn should_stake_and_unstake_queue() {
  new_test_ext().execute_with(|| {
    Context::default().mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS);

    assert_ok!(TidefiStaking::stake(
      Origin::signed(ALICE_ACCOUNT_ID),
      CurrencyId::Tdfy,
      ALICE_STAKE_ONE_TDFY,
      FIFTEEN_DAYS
    ));

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &ALICE_ACCOUNT_ID),
      ALICE_INITIAL_ONE_THOUSAND_TDFYS - ALICE_STAKE_ONE_TDFY
    );

    let stake_id = TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
      .first()
      .unwrap()
      .unique_id;

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tdfy),
      Some(ALICE_STAKE_ONE_TDFY)
    );

    // make sure the staking has been recorded in the storage
    assert!(TidefiStaking::account_stakes(ALICE_ACCOUNT_ID).len() == 1);
    assert!(
      TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
        .first()
        .unwrap()
        .initial_balance
        == ALICE_STAKE_ONE_TDFY
    );

    set_current_block(FIFTEEN_DAYS - 1_000);
    assert_eq!(Security::current_block_number(), FIFTEEN_DAYS - 1_000);

    assert_ok!(TidefiStaking::unstake(
      Origin::signed(ALICE_ACCOUNT_ID),
      stake_id,
      true
    ));

    assert!(TidefiStaking::account_stakes(ALICE_ACCOUNT_ID).len() == 1);
    let unstake_fee = TidefiStaking::unstake_fee() * ALICE_STAKE_ONE_TDFY;
    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &ALICE_ACCOUNT_ID),
      ALICE_INITIAL_ONE_THOUSAND_TDFYS - ALICE_STAKE_ONE_TDFY - unstake_fee
    );
    // 1 % of 1_000_000_000_000 = 10_000_000_000
    assert_eq!(unstake_fee, 10_000_000_000);

    // BlocksForceUnstake is set to 10, so let skip at least 10 blocks
    set_current_block(FIFTEEN_DAYS - 1_000 + BLOCKS_FORCE_UNLOCK + 1);
    assert_eq!(
      Security::current_block_number(),
      FIFTEEN_DAYS - 1_000 + BLOCKS_FORCE_UNLOCK + 1
    );

    assert!(TidefiStaking::unstake_queue().len() == 1);

    run_on_idle_hook(1, 1_000 * ONE_TDFY);

    assert!(TidefiStaking::unstake_queue().len() == 0);
  });
}

#[test]
pub fn should_stake_multiple_and_unstake_queue() {
  new_test_ext().execute_with(|| {
    Context::default()
      .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
      .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_ONE_THOUSAND_TDFYS);

    set_current_block(1);

    assert_ok!(TidefiStaking::stake(
      Origin::signed(ALICE_ACCOUNT_ID),
      CurrencyId::Tdfy,
      ALICE_STAKE_ONE_TDFY,
      FIFTEEN_DAYS
    ));

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &ALICE_ACCOUNT_ID),
      ALICE_INITIAL_ONE_THOUSAND_TDFYS - ALICE_STAKE_ONE_TDFY
    );

    let stake_id = TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
      .first()
      .unwrap()
      .unique_id;

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tdfy),
      Some(ALICE_STAKE_ONE_TDFY)
    );

    // make sure the staking has been recorded in the storage
    assert!(TidefiStaking::account_stakes(ALICE_ACCOUNT_ID).len() == 1);
    assert!(
      TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
        .first()
        .unwrap()
        .initial_balance
        == ALICE_STAKE_ONE_TDFY
    );

    set_current_block(FIFTEEN_DAYS - 3_000);
    assert_eq!(Security::current_block_number(), FIFTEEN_DAYS - 3_000);

    assert_ok!(TidefiStaking::stake(
      Origin::signed(BOB_ACCOUNT_ID),
      CurrencyId::Tdfy,
      BOB_STAKE_QUARTER_TDFY,
      FIFTEEN_DAYS
    ));

    let bob_stake_id = TidefiStaking::account_stakes(BOB_ACCOUNT_ID)
      .first()
      .unwrap()
      .unique_id;

    assert_ok!(TidefiStaking::stake(
      Origin::signed(BOB_ACCOUNT_ID),
      CurrencyId::Tdfy,
      BOB_STAKE_QUARTER_TDFY,
      FIFTEEN_DAYS * 2
    ));

    set_current_block(FIFTEEN_DAYS - 2_000);
    assert_eq!(Security::current_block_number(), FIFTEEN_DAYS - 2_000);

    assert_ok!(TidefiStaking::unstake(
      Origin::signed(BOB_ACCOUNT_ID),
      bob_stake_id,
      true
    ));

    set_current_block(FIFTEEN_DAYS - 2_000 + (BLOCKS_FORCE_UNLOCK / 2));
    assert_eq!(
      Security::current_block_number(),
      FIFTEEN_DAYS - 2_000 + (BLOCKS_FORCE_UNLOCK / 2)
    );
    assert_eq!(TidefiStaking::unstake_queue().len(), 1);

    assert_ok!(TidefiStaking::unstake(
      Origin::signed(ALICE_ACCOUNT_ID),
      stake_id,
      true
    ));
    assert_eq!(TidefiStaking::unstake_queue().len(), 2);

    assert_eq!(TidefiStaking::account_stakes(ALICE_ACCOUNT_ID).len(), 1);
    assert_eq!(TidefiStaking::account_stakes(BOB_ACCOUNT_ID).len(), 2);

    let unstake_fee = TidefiStaking::unstake_fee() * ALICE_STAKE_ONE_TDFY;
    let unstake_fee_bob = TidefiStaking::unstake_fee() * BOB_STAKE_QUARTER_TDFY;

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &ALICE_ACCOUNT_ID),
      ALICE_INITIAL_ONE_THOUSAND_TDFYS - ALICE_STAKE_ONE_TDFY - unstake_fee
    );

    // 1 % of 1_000_000_000_000 = 10_000_000_000
    assert_eq!(unstake_fee, 10_000_000_000);

    // BlocksForceUnstake is set to 10, so let skip at least 10 blocks
    set_current_block(FIFTEEN_DAYS - 2_000 + BLOCKS_FORCE_UNLOCK + 1);
    assert_eq!(
      Security::current_block_number(),
      FIFTEEN_DAYS - 2_000 + BLOCKS_FORCE_UNLOCK + 1
    );

    assert_eq!(TidefiStaking::unstake_queue().len(), 2);

    run_on_idle_hook(1, 1_000 * ONE_TDFY);

    assert_eq!(TidefiStaking::unstake_queue().len(), 1);

    set_current_block(FIFTEEN_DAYS - 2_000 + BLOCKS_FORCE_UNLOCK + (BLOCKS_FORCE_UNLOCK / 2) + 1);
    assert_eq!(
      Security::current_block_number(),
      FIFTEEN_DAYS - 2_000 + BLOCKS_FORCE_UNLOCK + (BLOCKS_FORCE_UNLOCK / 2) + 1
    );

    run_on_idle_hook(1, 1_000 * ONE_TDFY);

    assert!(TidefiStaking::unstake_queue().is_empty());

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &ALICE_ACCOUNT_ID),
      ALICE_INITIAL_ONE_THOUSAND_TDFYS - unstake_fee
    );

    assert!(TidefiStaking::account_stakes(BOB_ACCOUNT_ID).len() == 1);
    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
      // we still have a stake active
      BOB_INITIAL_ONE_THOUSAND_TDFYS - unstake_fee_bob - BOB_STAKE_QUARTER_TDFY
    );
  });
}

#[test]
pub fn should_calculate_rewards() {
  new_test_ext().execute_with(|| {
    const ALICE_STAKE_ONE_HUNDRED_TDFYS: Balance = 100 * ONE_TDFY;
    const BOB_STAKE_ONE_HUNDRED_TDFYS: Balance = 100 * ONE_TDFY;
    const CHARLIE_STAKE_FOUR_HUNDRED_TDFYS: Balance = 400 * ONE_TDFY;
    const SESSION_TRADE_VALUE_ONE_HUNDRED_TDFYS: Balance = 100 * ONE_TDFY;

    Context::default()
      .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
      .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_ONE_THOUSAND_TDFYS)
      .mint_tdfy(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_ONE_THOUSAND_TDFYS);

    set_current_block(1);

    assert_ok!(TidefiStaking::stake(
      Origin::signed(ALICE_ACCOUNT_ID),
      CurrencyId::Tdfy,
      ALICE_STAKE_ONE_HUNDRED_TDFYS,
      FIFTEEN_DAYS
    ));

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tdfy),
      Some(ALICE_STAKE_ONE_HUNDRED_TDFYS)
    );

    // make sure the staking has been recorded in the storage
    assert!(TidefiStaking::account_stakes(ALICE_ACCOUNT_ID).len() == 1);
    assert_eq!(
      TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
        .first()
        .unwrap()
        .initial_balance,
      ALICE_STAKE_ONE_HUNDRED_TDFYS
    );

    // 100 for TDFY in fees for session 1
    // 15 days should get 2%, so 2 tides
    assert_ok!(TidefiStaking::on_session_end(
      1,
      vec![(CurrencyId::Tdfy, SESSION_TRADE_VALUE_ONE_HUNDRED_TDFYS)]
    ));

    run_on_idle_hook(1, 1_000 * ONE_TDFY);

    let alice_staked_tdfy_principal_after_session_1 = ALICE_STAKE_ONE_HUNDRED_TDFYS + 2 * ONE_TDFY;

    assert_eq!(
      TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      alice_staked_tdfy_principal_after_session_1
    );

    assert_ok!(TidefiStaking::stake(
      Origin::signed(BOB_ACCOUNT_ID),
      CurrencyId::Tdfy,
      BOB_STAKE_ONE_HUNDRED_TDFYS,
      FIFTEEN_DAYS
    ));

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tdfy),
      Some(ALICE_STAKE_ONE_HUNDRED_TDFYS + BOB_STAKE_ONE_HUNDRED_TDFYS)
    );

    // 100 for TDFY in fees for session 1
    // 15 days should get 2%, so 2 tides
    assert_ok!(TidefiStaking::on_session_end(
      2,
      vec![(CurrencyId::Tdfy, SESSION_TRADE_VALUE_ONE_HUNDRED_TDFYS)]
    ));

    run_on_idle_hook(1, 1_000 * ONE_TDFY);

    let alice_staked_tdfy_principal_after_session_2 =
      alice_staked_tdfy_principal_after_session_1 + ONE_TDFY;
    let bob_staked_tdfy_principal_after_session_2 = BOB_STAKE_ONE_HUNDRED_TDFYS + ONE_TDFY;

    assert_eq!(
      TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      alice_staked_tdfy_principal_after_session_2
    );

    assert_eq!(
      TidefiStaking::account_stakes(BOB_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      bob_staked_tdfy_principal_after_session_2
    );

    // 2 empty sessions
    assert_ok!(TidefiStaking::on_session_end(3, Vec::new()));
    assert_ok!(TidefiStaking::on_session_end(4, Vec::new()));
    run_on_idle_hook(1, 1_000 * ONE_TDFY);

    assert_eq!(
      TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      alice_staked_tdfy_principal_after_session_2
    );

    assert_eq!(
      TidefiStaking::account_stakes(BOB_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      bob_staked_tdfy_principal_after_session_2
    );

    assert_ok!(TidefiStaking::stake(
      Origin::signed(CHARLIE_ACCOUNT_ID),
      CurrencyId::Tdfy,
      CHARLIE_STAKE_FOUR_HUNDRED_TDFYS,
      FIFTEEN_DAYS
    ));

    assert_ok!(TidefiStaking::on_session_end(
      5,
      vec![(CurrencyId::Tdfy, SESSION_TRADE_VALUE_ONE_HUNDRED_TDFYS)]
    ));

    run_on_idle_hook(1, 1_000 * ONE_TDFY);

    let total_staked_tdfys_after_session_5 = ALICE_STAKE_ONE_HUNDRED_TDFYS
      .saturating_add(BOB_STAKE_ONE_HUNDRED_TDFYS)
      .saturating_add(CHARLIE_STAKE_FOUR_HUNDRED_TDFYS);
    let total_stake_rewards_after_session_5 = 2 * ONE_TDFY;

    assert_eq!(
      TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      alice_staked_tdfy_principal_after_session_2.saturating_add(
        ALICE_STAKE_ONE_HUNDRED_TDFYS
          .saturating_mul(total_stake_rewards_after_session_5)
          .saturating_div(total_staked_tdfys_after_session_5)
      )
    );

    assert_eq!(
      TidefiStaking::account_stakes(BOB_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      bob_staked_tdfy_principal_after_session_2.saturating_add(
        BOB_STAKE_ONE_HUNDRED_TDFYS
          .saturating_mul(total_stake_rewards_after_session_5)
          .saturating_div(total_staked_tdfys_after_session_5)
      )
    );

    assert_eq!(
      TidefiStaking::account_stakes(CHARLIE_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      CHARLIE_STAKE_FOUR_HUNDRED_TDFYS.saturating_add(
        CHARLIE_STAKE_FOUR_HUNDRED_TDFYS
          .saturating_mul(total_stake_rewards_after_session_5)
          .saturating_div(total_staked_tdfys_after_session_5)
      )
    );
  });
}
