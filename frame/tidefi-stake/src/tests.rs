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
    new_test_ext, AccountId, Adapter, Fees, Oracle, RuntimeEvent as MockEvent, RuntimeOrigin,
    Security, StakeAccountCap, System, Test, Tidefi, TidefiStaking,
  },
  pallet as pallet_tidefi_stake, AccountStakes, Error, PendingStoredSessions, QueueUnstake,
  SessionTotalFees, StakingPool,
};
use frame_support::{
  assert_noop, assert_ok,
  traits::{
    fungibles::{Inspect, InspectHold, Mutate},
    Get, Hooks,
  },
  BoundedVec,
};
use sp_runtime::{
  traits::{AccountIdConversion, BadOrigin},
  ArithmeticError, DispatchError, Percent, Permill, Perquintill,
};
use std::str::FromStr;
use tidefi_primitives::{
  pallet::{FeesExt, OracleExt, StakingExt},
  Balance, CurrencyId, Hash, Stake, StakeStatus, SwapConfirmation, SwapStatus, SwapType,
};

type BlockNumber = u64;

const TEST_TOKEN: u32 = 2;
const TEST_TOKEN_CURRENCY_ID: CurrencyId = CurrencyId::Wrapped(TEST_TOKEN);
const FIFTEEN_DAYS: BlockNumber = 14400 * 15;
const BLOCKS_FORCE_UNLOCK: BlockNumber = 256;

// Extrinsic Hashes
const EXTRINSIC_HASH_0: [u8; 32] = [0; 32];
const EXTRINSIC_HASH_1: [u8; 32] = [1; 32];

// Slippage Rates
const SLIPPAGE_0_PERCENT: Permill = Permill::from_percent(0);

// Asset Units
const ONE_TDFY: Balance = 1_000_000_000_000;
const ONE_TEST_TOKEN: Balance = 100_000_000;

// Test Accounts
const ALICE_ACCOUNT_ID: AccountId = AccountId(1);
const BOB_ACCOUNT_ID: AccountId = AccountId(2);
const CHARLIE_ACCOUNT_ID: AccountId = AccountId(3);
const OPERATOR_ACCOUNT_ID: AccountId = AccountId(100);

const ALICE_INITIAL_ONE_THOUSAND_TDFYS: Balance = 1_000 * ONE_TDFY;
const ALICE_INITIAL_ONE_THOUSAND_TEST_TOKENS: Balance = 1_000 * ONE_TEST_TOKEN;
const BOB_INITIAL_ONE_THOUSAND_TDFYS: Balance = 1_000 * ONE_TDFY;
const BOB_INITIAL_ONE_THOUSAND_TEST_TOKENS: Balance = 1_000 * ONE_TEST_TOKEN;
const CHARLIE_INITIAL_ONE_THOUSAND_TDFYS: Balance = 1_000 * ONE_TDFY;
const CHARLIE_INITIAL_ONE_THOUSAND_TEST_TOKENS: Balance = 1_000 * ONE_TEST_TOKEN;

const ALICE_STAKE_ONE_TDFY: Balance = ONE_TDFY;
const BOB_STAKE_QUARTER_TDFY: Balance = ALICE_STAKE_ONE_TDFY / 4;

const BLOCK_NUMBER_ZERO: BlockNumber = 0;

const SESSION_TOTAL_FEES_ONE_HUNDRED_TEST_TOKENS: Balance = 100 * ONE_TEST_TOKEN;

#[derive(Clone)]
struct Context {
  staker: AccountId,
  staking_pallet_account: AccountId,
  fees_pallet_account: AccountId,
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
      fees_pallet_account: <Test as pallet_fees::Config>::FeesPalletId::get()
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
    assert_ok!(Oracle::set_status(
      RuntimeOrigin::signed(ALICE_ACCOUNT_ID),
      status
    ));
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

  fn set_operator_account(self) -> Self {
    Self::mint_tdfy(self.clone(), OPERATOR_ACCOUNT_ID, ONE_TDFY);
    let _ = TidefiStaking::set_operator_account_id(RuntimeOrigin::root(), OPERATOR_ACCOUNT_ID);
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
      RuntimeOrigin::signed(self.staker),
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
      RuntimeOrigin::signed(self.staker),
      TEST_TOKEN_CURRENCY_ID,
      self.test_token_amount,
      self.duration
    ));

    assert_eq!(
      self.test_token_amount,
      Adapter::balance(TEST_TOKEN_CURRENCY_ID, &self.staking_pallet_account)
    );

    assert_eq!(
      TidefiStaking::staking_pool(TEST_TOKEN_CURRENCY_ID),
      Some(self.test_token_amount)
    );

    assert!(
      TidefiStaking::account_stakes(self.staker)
        .first()
        .unwrap()
        .initial_balance
        == self.test_token_amount
    );

    assert!(
      TidefiStaking::account_stakes(self.staker)
        .first()
        .unwrap()
        .principal
        == self.test_token_amount
    );

    self
  }

  fn add_mock_account_stakes(self, account_id: AccountId, number_of_stakes: usize) -> Self {
    AccountStakes::<Test>::insert(
      account_id,
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

#[allow(clippy::too_many_arguments)]
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
    swap_type.clone(),
    slippage,
  )
  .unwrap();

  let swap_fee =
    Fees::calculate_swap_fees(asset_id_from, amount_from, swap_type, is_market_maker).fee;

  assert_eq!(
    Adapter::balance_on_hold(asset_id_from, &account_id),
    amount_from.saturating_add(swap_fee)
  );

  if asset_id_from != CurrencyId::Tdfy {
    assert_eq!(
      Adapter::balance(asset_id_from, &account_id),
      initial_from_token_balance
        .saturating_sub(amount_from)
        .saturating_sub(swap_fee)
    );
  }

  assert_spendable_balance_is_updated(
    account_id,
    asset_id_from,
    initial_from_token_balance,
    amount_from,
    swap_fee,
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

fn assert_spendable_balance_is_updated(
  account_id: AccountId,
  currency_id: CurrencyId,
  initial_balance: Balance,
  sell_amount: Balance,
  swap_fee: Balance,
) {
  let expected_reducible_balance = initial_balance
    .saturating_sub(sell_amount)
    .saturating_sub(swap_fee);

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
      .saturating_sub(swap_fee)
  );
}

fn set_current_block(block_number: BlockNumber) {
  <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
    *n = block_number;
    *n
  });
}

fn run_on_idle_hook<T: pallet_tidefi_stake::Config>() {
  let remaining_weight = <T as frame_system::Config>::BlockWeights::get().max_block;
  TidefiStaking::on_idle(0, remaining_weight);
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
          RuntimeOrigin::signed(context.staker),
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
          RuntimeOrigin::signed(context.staker),
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
            RuntimeOrigin::none(),
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
            RuntimeOrigin::signed(context.staker),
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

        let minimum_stake_amount = TidefiStaking::staking_meta(TEST_TOKEN_CURRENCY_ID)
          .unwrap()
          .minimum_amount;
        assert_noop!(
          TidefiStaking::stake(
            RuntimeOrigin::signed(context.staker),
            TEST_TOKEN_CURRENCY_ID,
            minimum_stake_amount - 1,
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
            RuntimeOrigin::signed(context.staker),
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
            RuntimeOrigin::signed(context.staker),
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
            RuntimeOrigin::signed(context.staker),
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
          let context = Context::default();
          let fees_pallet_account_initial_tdfy_balance =
            Adapter::balance(CurrencyId::Tdfy, &context.fees_pallet_account);
          let fees_pallet_account_initial_test_token_balance =
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.fees_pallet_account);
          let staking_pallet_account_initial_tdfy_balance =
            Adapter::balance(CurrencyId::Tdfy, &context.staking_pallet_account);
          const SWAP_TDFY_AMOUNT: Balance = 10 * ONE_TDFY;
          const SWAP_TEST_TOKEN_AMOUNT: Balance = 2 * ONE_TEST_TOKEN;

          let context = Context::default()
            .set_oracle_status(true)
            .set_market_makers(vec![ALICE_ACCOUNT_ID, CHARLIE_ACCOUNT_ID])
            .set_operator_account()
            .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
            .mint_test_token(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TEST_TOKENS)
            .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_ONE_THOUSAND_TDFYS)
            .mint_test_token(BOB_ACCOUNT_ID, BOB_INITIAL_ONE_THOUSAND_TEST_TOKENS)
            .mint_tdfy(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_ONE_THOUSAND_TDFYS)
            .mint_test_token(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_ONE_THOUSAND_TEST_TOKENS)
            .stake_tdfy();

          let staker_tdfys_before_unstaking = Adapter::balance(CurrencyId::Tdfy, &context.staker);

          // Create Swap Requests
          let trade_request_id_1 = context.create_tdfy_to_temp_limit_swap_request(
            BOB_ACCOUNT_ID,
            SWAP_TDFY_AMOUNT,
            SWAP_TEST_TOKEN_AMOUNT,
            EXTRINSIC_HASH_0,
            SLIPPAGE_0_PERCENT,
          );

          let trade_request_id_2 = context.create_temp_to_tdfy_limit_swap_request(
            CHARLIE_ACCOUNT_ID,
            SWAP_TEST_TOKEN_AMOUNT,
            SWAP_TDFY_AMOUNT,
            EXTRINSIC_HASH_1,
            SLIPPAGE_0_PERCENT,
          );

          assert!(Oracle::swaps(trade_request_id_1).is_some());
          assert!(Oracle::swaps(trade_request_id_2).is_some());

          // Confirm Swaps
          assert_ok!(Oracle::confirm_swap(
            Oracle::account_id().into(),
            trade_request_id_1,
            vec![SwapConfirmation {
              request_id: trade_request_id_2,
              amount_to_receive: SWAP_TDFY_AMOUNT,
              amount_to_send: SWAP_TEST_TOKEN_AMOUNT
            },],
          ));

          assert!(Oracle::swaps(trade_request_id_1).is_none());
          assert!(Oracle::swaps(trade_request_id_2).is_none());

          System::assert_has_event(MockEvent::Oracle(pallet_oracle::Event::SwapProcessed {
            request_id: trade_request_id_1,
            status: SwapStatus::Completed,
            account_id: BOB_ACCOUNT_ID,
            currency_from: CurrencyId::Tdfy,
            currency_amount_from: SWAP_TDFY_AMOUNT,
            currency_to: TEST_TOKEN_CURRENCY_ID,
            currency_amount_to: SWAP_TEST_TOKEN_AMOUNT,
            initial_extrinsic_hash: EXTRINSIC_HASH_0,
          }));

          System::assert_has_event(MockEvent::Oracle(pallet_oracle::Event::SwapProcessed {
            request_id: trade_request_id_2,
            status: SwapStatus::Completed,
            account_id: CHARLIE_ACCOUNT_ID,
            currency_from: TEST_TOKEN_CURRENCY_ID,
            currency_amount_from: SWAP_TEST_TOKEN_AMOUNT,
            currency_to: CurrencyId::Tdfy,
            currency_amount_to: SWAP_TDFY_AMOUNT,
            initial_extrinsic_hash: EXTRINSIC_HASH_1,
          }));

          // Calculate expected fees in both TDFYs and test tokens
          let total_fee_in_tdfy =
            Fees::calculate_swap_fees(CurrencyId::Tdfy, SWAP_TDFY_AMOUNT, SwapType::Limit, false)
              .fee;
          let total_fee_in_test_token = Fees::calculate_swap_fees(
            TEST_TOKEN_CURRENCY_ID,
            SWAP_TEST_TOKEN_AMOUNT,
            SwapType::Limit,
            true,
          )
          .fee;

          // Assert fees in both TDFYs and test tokens are paid to fees pallet account
          assert_eq!(
            fees_pallet_account_initial_tdfy_balance + total_fee_in_tdfy,
            Adapter::balance(CurrencyId::Tdfy, &context.fees_pallet_account)
          );
          assert_eq!(
            fees_pallet_account_initial_test_token_balance + total_fee_in_test_token,
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.fees_pallet_account)
          );

          // End the session in an advanced block, so the session total fees is transferred from fees pallet to operator account in stake pallet
          set_current_block(2);
          assert_ok!(TidefiStaking::on_session_end(
            2,
            vec![
              (
                CurrencyId::Tdfy,
                // fees pallet account received fees
                Adapter::balance(CurrencyId::Tdfy, &context.fees_pallet_account)
                  - fees_pallet_account_initial_tdfy_balance
              ),
              (
                TEST_TOKEN_CURRENCY_ID,
                Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.fees_pallet_account)
              )
            ],
            context.fees_pallet_account,
          ));

          // Finish staking period
          set_current_block(FIFTEEN_DAYS + 1);

          // Get stake from chain storage
          let stake = AccountStakes::<Test>::get(context.staker)
            .into_iter()
            .find(|stake| stake.unique_id == context.stake_id)
            .unwrap();
          let latest_principal_with_rewards = stake.principal;

          assert_ok!(TidefiStaking::unstake(
            RuntimeOrigin::signed(context.staker),
            context.stake_id,
            false
          ));
          System::assert_has_event(MockEvent::TidefiStaking(
            pallet_tidefi_stake::Event::Unstaked {
              request_id: context.stake_id,
              account_id: context.staker,
              currency_id: CurrencyId::Tdfy,
              initial_balance: context.tdfy_amount,
              final_balance: latest_principal_with_rewards,
            },
          ));

          // The only staker receives 100% of the session total fees as a reward
          assert_eq!(
            staker_tdfys_before_unstaking + latest_principal_with_rewards,
            Adapter::balance(CurrencyId::Tdfy, &context.staker)
          );

          // Staking pallet account tdfy balance equals to its initial value
          assert_eq!(
            staking_pallet_account_initial_tdfy_balance,
            Adapter::balance(CurrencyId::Tdfy, &context.staking_pallet_account)
          );

          // Fee pallet account tdfy balance equals to its initial value
          assert_eq!(
            fees_pallet_account_initial_tdfy_balance,
            Adapter::balance(CurrencyId::Tdfy, &context.fees_pallet_account)
          );

          // Assert Bob TDFY and Test token balance
          assert_eq!(
            BOB_INITIAL_ONE_THOUSAND_TDFYS - SWAP_TDFY_AMOUNT - total_fee_in_tdfy,
            Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID)
          );
          assert_eq!(
            BOB_INITIAL_ONE_THOUSAND_TEST_TOKENS + SWAP_TEST_TOKEN_AMOUNT,
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &BOB_ACCOUNT_ID)
          );
          // Assert Charlie TDFY and Test token balance
          assert_eq!(
            CHARLIE_INITIAL_ONE_THOUSAND_TDFYS + SWAP_TDFY_AMOUNT,
            Adapter::balance(CurrencyId::Tdfy, &CHARLIE_ACCOUNT_ID)
          );
          assert_eq!(
            CHARLIE_INITIAL_ONE_THOUSAND_TEST_TOKENS
              - SWAP_TEST_TOKEN_AMOUNT
              - total_fee_in_test_token,
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &CHARLIE_ACCOUNT_ID)
          );
        });
      }

      #[test]
      fn for_wrapped_asset() {
        new_test_ext().execute_with(|| {
          let context = Context::default();
          let fees_pallet_account_initial_tdfy_balance =
            Adapter::balance(CurrencyId::Tdfy, &context.fees_pallet_account);
          let fees_pallet_account_initial_test_token_balance =
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.fees_pallet_account);
          let staking_pallet_account_initial_test_token_balance =
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.staking_pallet_account);
          const SWAP_TDFY_AMOUNT: Balance = 10 * ONE_TDFY;
          const SWAP_TEST_TOKEN_AMOUNT: Balance = 2 * ONE_TEST_TOKEN;

          // Mint and Stake
          let context = Context::default()
            .set_oracle_status(true)
            .set_market_makers(vec![ALICE_ACCOUNT_ID, CHARLIE_ACCOUNT_ID])
            .set_operator_account()
            .mint_tdfy(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TDFYS)
            .mint_test_token(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TEST_TOKENS)
            .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_ONE_THOUSAND_TDFYS)
            .mint_test_token(BOB_ACCOUNT_ID, BOB_INITIAL_ONE_THOUSAND_TEST_TOKENS)
            .mint_tdfy(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_ONE_THOUSAND_TDFYS)
            .mint_test_token(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_ONE_THOUSAND_TEST_TOKENS)
            .stake_test_tokens();

          let staker_test_tokens_before_unstaking =
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.staker);

          // Create Swap Requests
          let trade_request_id_1 = context.create_tdfy_to_temp_limit_swap_request(
            BOB_ACCOUNT_ID,
            SWAP_TDFY_AMOUNT,
            SWAP_TEST_TOKEN_AMOUNT,
            EXTRINSIC_HASH_0,
            SLIPPAGE_0_PERCENT,
          );

          let trade_request_id_2 = context.create_temp_to_tdfy_limit_swap_request(
            CHARLIE_ACCOUNT_ID,
            SWAP_TEST_TOKEN_AMOUNT,
            SWAP_TDFY_AMOUNT,
            EXTRINSIC_HASH_1,
            SLIPPAGE_0_PERCENT,
          );

          assert!(Oracle::swaps(trade_request_id_1).is_some());
          assert!(Oracle::swaps(trade_request_id_2).is_some());

          // Confirm Swaps
          assert_ok!(Oracle::confirm_swap(
            Oracle::account_id().into(),
            trade_request_id_1,
            vec![SwapConfirmation {
              request_id: trade_request_id_2,
              amount_to_receive: SWAP_TDFY_AMOUNT,
              amount_to_send: SWAP_TEST_TOKEN_AMOUNT
            },],
          ));

          assert!(Oracle::swaps(trade_request_id_1).is_none());
          assert!(Oracle::swaps(trade_request_id_2).is_none());

          System::assert_has_event(MockEvent::Oracle(pallet_oracle::Event::SwapProcessed {
            request_id: trade_request_id_1,
            status: SwapStatus::Completed,
            account_id: BOB_ACCOUNT_ID,
            currency_from: CurrencyId::Tdfy,
            currency_amount_from: SWAP_TDFY_AMOUNT,
            currency_to: TEST_TOKEN_CURRENCY_ID,
            currency_amount_to: SWAP_TEST_TOKEN_AMOUNT,
            initial_extrinsic_hash: EXTRINSIC_HASH_0,
          }));

          System::assert_has_event(MockEvent::Oracle(pallet_oracle::Event::SwapProcessed {
            request_id: trade_request_id_2,
            status: SwapStatus::Completed,
            account_id: CHARLIE_ACCOUNT_ID,
            currency_from: TEST_TOKEN_CURRENCY_ID,
            currency_amount_from: SWAP_TEST_TOKEN_AMOUNT,
            currency_to: CurrencyId::Tdfy,
            currency_amount_to: SWAP_TDFY_AMOUNT,
            initial_extrinsic_hash: EXTRINSIC_HASH_1,
          }));

          // Calculate expected fees in both TDFYs and test tokens
          let total_fee_in_tdfy =
            Fees::calculate_swap_fees(CurrencyId::Tdfy, SWAP_TDFY_AMOUNT, SwapType::Limit, false)
              .fee;
          let total_fee_in_test_token = Fees::calculate_swap_fees(
            TEST_TOKEN_CURRENCY_ID,
            SWAP_TEST_TOKEN_AMOUNT,
            SwapType::Limit,
            true,
          )
          .fee;

          // Assert fees in both TDFYs and test tokens are paid to fees pallet account
          assert_eq!(
            fees_pallet_account_initial_tdfy_balance + total_fee_in_tdfy,
            Adapter::balance(CurrencyId::Tdfy, &context.fees_pallet_account)
          );
          assert_eq!(
            fees_pallet_account_initial_test_token_balance + total_fee_in_test_token,
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.fees_pallet_account)
          );

          // End the session in an advanced block, so the session total fees is transferred from fees pallet to operator account in stake pallet
          set_current_block(2);
          assert_ok!(TidefiStaking::on_session_end(
            2,
            vec![
              (
                CurrencyId::Tdfy,
                Adapter::balance(CurrencyId::Tdfy, &context.fees_pallet_account)
              ),
              (
                TEST_TOKEN_CURRENCY_ID,
                Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.fees_pallet_account)
              )
            ],
            context.fees_pallet_account,
          ));

          // run 2x on_idle to clean the Queue
          run_on_idle_hook::<Test>();
          run_on_idle_hook::<Test>();

          // Finish staking period
          set_current_block(FIFTEEN_DAYS + 1);

          // Get stake from chain storage
          let stake = AccountStakes::<Test>::get(context.staker)
            .into_iter()
            .find(|stake| stake.unique_id == context.stake_id)
            .unwrap();
          let latest_principal_with_rewards = stake.principal;

          // Unstake
          assert_ok!(TidefiStaking::unstake(
            RuntimeOrigin::signed(context.staker),
            context.stake_id,
            false
          ));
          System::assert_has_event(MockEvent::TidefiStaking(
            pallet_tidefi_stake::Event::Unstaked {
              request_id: context.stake_id,
              account_id: context.staker,
              currency_id: TEST_TOKEN_CURRENCY_ID,
              initial_balance: context.test_token_amount,
              final_balance: latest_principal_with_rewards,
            },
          ));

          // The only staker receives 100% of the session total fees as a reward
          assert_eq!(
            staker_test_tokens_before_unstaking + latest_principal_with_rewards,
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.staker)
          );

          // Operator account keeps the rest of the session total fees
          assert_eq!(
            total_fee_in_test_token - (latest_principal_with_rewards - context.test_token_amount),
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &OPERATOR_ACCOUNT_ID)
          );

          // Staking pallet account test token balance equals to its initial value
          assert_eq!(
            staking_pallet_account_initial_test_token_balance,
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.staking_pallet_account)
          );

          // Fee pallet account test token balance equals to its initial value
          assert_eq!(
            fees_pallet_account_initial_test_token_balance,
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &context.fees_pallet_account)
          );

          // Assert Bob TDFY and Test token balance
          assert_eq!(
            BOB_INITIAL_ONE_THOUSAND_TDFYS - SWAP_TDFY_AMOUNT - total_fee_in_tdfy,
            Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID)
          );
          assert_eq!(
            BOB_INITIAL_ONE_THOUSAND_TEST_TOKENS + SWAP_TEST_TOKEN_AMOUNT,
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &BOB_ACCOUNT_ID)
          );
          // Assert Charlie TDFY and Test token balance
          assert_eq!(
            CHARLIE_INITIAL_ONE_THOUSAND_TDFYS + SWAP_TDFY_AMOUNT,
            Adapter::balance(CurrencyId::Tdfy, &CHARLIE_ACCOUNT_ID)
          );
          assert_eq!(
            CHARLIE_INITIAL_ONE_THOUSAND_TEST_TOKENS
              - SWAP_TEST_TOKEN_AMOUNT
              - total_fee_in_test_token,
            Adapter::balance(TEST_TOKEN_CURRENCY_ID, &CHARLIE_ACCOUNT_ID)
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
            RuntimeOrigin::signed(context.staker),
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
            RuntimeOrigin::signed(context.staker),
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
          TidefiStaking::unstake(RuntimeOrigin::none(), Hash::zero(), true),
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
          TidefiStaking::unstake(RuntimeOrigin::signed(context.staker), Hash::zero(), true),
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
          TidefiStaking::unstake(
            RuntimeOrigin::signed(context.staker),
            context.stake_id,
            false
          ),
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
          TidefiStaking::unstake(
            RuntimeOrigin::signed(context.staker),
            context.stake_id,
            true
          ),
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
          TidefiStaking::unstake(
            RuntimeOrigin::signed(context.staker),
            context.stake_id,
            true
          ),
          Error::<Test>::InsufficientBalance
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
      RuntimeOrigin::signed(ALICE_ACCOUNT_ID),
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
      RuntimeOrigin::signed(ALICE_ACCOUNT_ID),
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
      RuntimeOrigin::signed(ALICE_ACCOUNT_ID),
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
      RuntimeOrigin::signed(ALICE_ACCOUNT_ID),
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

    assert!(QueueUnstake::<Test>::count() == 1);

    run_on_idle_hook::<Test>();

    assert!(QueueUnstake::<Test>::count() == 0);
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
      RuntimeOrigin::signed(ALICE_ACCOUNT_ID),
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
      RuntimeOrigin::signed(BOB_ACCOUNT_ID),
      CurrencyId::Tdfy,
      BOB_STAKE_QUARTER_TDFY,
      FIFTEEN_DAYS
    ));

    let bob_stake_id = TidefiStaking::account_stakes(BOB_ACCOUNT_ID)
      .first()
      .unwrap()
      .unique_id;

    assert_ok!(TidefiStaking::stake(
      RuntimeOrigin::signed(BOB_ACCOUNT_ID),
      CurrencyId::Tdfy,
      BOB_STAKE_QUARTER_TDFY,
      FIFTEEN_DAYS * 2
    ));

    set_current_block(FIFTEEN_DAYS - 2_000);
    assert_eq!(Security::current_block_number(), FIFTEEN_DAYS - 2_000);

    assert_ok!(TidefiStaking::unstake(
      RuntimeOrigin::signed(BOB_ACCOUNT_ID),
      bob_stake_id,
      true
    ));

    set_current_block(FIFTEEN_DAYS - 2_000 + (BLOCKS_FORCE_UNLOCK / 2));
    assert_eq!(
      Security::current_block_number(),
      FIFTEEN_DAYS - 2_000 + (BLOCKS_FORCE_UNLOCK / 2)
    );
    assert_eq!(QueueUnstake::<Test>::count(), 1);

    assert_ok!(TidefiStaking::unstake(
      RuntimeOrigin::signed(ALICE_ACCOUNT_ID),
      stake_id,
      true
    ));
    assert_eq!(QueueUnstake::<Test>::count(), 2);

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

    assert_eq!(QueueUnstake::<Test>::count(), 2);

    run_on_idle_hook::<Test>();

    assert_eq!(QueueUnstake::<Test>::count(), 1);

    set_current_block(FIFTEEN_DAYS - 2_000 + BLOCKS_FORCE_UNLOCK + (BLOCKS_FORCE_UNLOCK / 2) + 1);
    assert_eq!(
      Security::current_block_number(),
      FIFTEEN_DAYS - 2_000 + BLOCKS_FORCE_UNLOCK + (BLOCKS_FORCE_UNLOCK / 2) + 1
    );

    run_on_idle_hook::<Test>();

    assert_eq!(QueueUnstake::<Test>::count(), 0);

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
    const ALICE_STAKE_ONE_HUNDRED_TEST_TOKENS: Balance = 100 * ONE_TEST_TOKEN;
    const BOB_STAKE_ONE_HUNDRED_TEST_TOKENS: Balance = 100 * ONE_TEST_TOKEN;
    const CHARLIE_STAKE_FOUR_HUNDRED_TEST_TOKENS: Balance = 400 * ONE_TEST_TOKEN;
    let fees_pallet_account: AccountId =
      <Test as pallet_fees::Config>::FeesPalletId::get().into_account_truncating();

    Context::default()
      .mint_test_token(ALICE_ACCOUNT_ID, ALICE_INITIAL_ONE_THOUSAND_TEST_TOKENS)
      .mint_test_token(BOB_ACCOUNT_ID, BOB_INITIAL_ONE_THOUSAND_TEST_TOKENS)
      .mint_test_token(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_ONE_THOUSAND_TEST_TOKENS);

    assert_ok!(Tidefi::transfer(
      RuntimeOrigin::signed(ALICE_ACCOUNT_ID),
      fees_pallet_account,
      TEST_TOKEN_CURRENCY_ID,
      500 * ONE_TEST_TOKEN
    ));

    set_current_block(1);

    assert_ok!(TidefiStaking::stake(
      RuntimeOrigin::signed(ALICE_ACCOUNT_ID),
      TEST_TOKEN_CURRENCY_ID,
      ALICE_STAKE_ONE_HUNDRED_TEST_TOKENS,
      FIFTEEN_DAYS
    ));

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(TEST_TOKEN_CURRENCY_ID),
      Some(ALICE_STAKE_ONE_HUNDRED_TEST_TOKENS)
    );

    // make sure the staking has been recorded in the storage
    assert!(TidefiStaking::account_stakes(ALICE_ACCOUNT_ID).len() == 1);
    assert_eq!(
      TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
        .first()
        .unwrap()
        .initial_balance,
      ALICE_STAKE_ONE_HUNDRED_TEST_TOKENS
    );

    // 100 for TDFY in fees for session 1
    // 15 days should get 2%, so 2 tides (100% of the pool)
    let expected_max_rewards = 2 * ONE_TEST_TOKEN;

    assert_ok!(TidefiStaking::on_session_end(
      1,
      vec![(
        TEST_TOKEN_CURRENCY_ID,
        SESSION_TOTAL_FEES_ONE_HUNDRED_TEST_TOKENS
      )],
      fees_pallet_account
    ));

    run_on_idle_hook::<Test>();

    // 100% of the pool (2 TDFY as reward)
    let alice_staked_principal_after_session_1 =
      ALICE_STAKE_ONE_HUNDRED_TEST_TOKENS.saturating_add(expected_max_rewards);

    assert_eq!(
      TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      alice_staked_principal_after_session_1
    );

    assert_ok!(TidefiStaking::stake(
      RuntimeOrigin::signed(BOB_ACCOUNT_ID),
      TEST_TOKEN_CURRENCY_ID,
      BOB_STAKE_ONE_HUNDRED_TEST_TOKENS,
      FIFTEEN_DAYS
    ));

    // make sure the staking pool has been updated after bob stake
    assert_eq!(
      TidefiStaking::staking_pool(TEST_TOKEN_CURRENCY_ID),
      Some(
        alice_staked_principal_after_session_1.saturating_add(BOB_STAKE_ONE_HUNDRED_TEST_TOKENS)
      )
    );

    assert_ok!(TidefiStaking::on_session_end(
      2,
      vec![(
        TEST_TOKEN_CURRENCY_ID,
        SESSION_TOTAL_FEES_ONE_HUNDRED_TEST_TOKENS
      )],
      fees_pallet_account
    ));

    let staking_pool_before_compound_session_1 =
      TidefiStaking::staking_pool(TEST_TOKEN_CURRENCY_ID).unwrap_or(0);
    run_on_idle_hook::<Test>();

    let alice_pool_percentage_after_session_2 = Perquintill::from_rational(
      alice_staked_principal_after_session_1,
      staking_pool_before_compound_session_1,
    );

    let expected_alice_staked_principal_after_session_2 = alice_staked_principal_after_session_1
      .saturating_add(alice_pool_percentage_after_session_2 * expected_max_rewards);

    let bob_pool_after_session_2 = Perquintill::from_rational(
      BOB_STAKE_ONE_HUNDRED_TEST_TOKENS,
      expected_alice_staked_principal_after_session_2
        .saturating_add(BOB_STAKE_ONE_HUNDRED_TEST_TOKENS),
    );

    let expected_bob_staked_principal_after_session_2 = BOB_STAKE_ONE_HUNDRED_TEST_TOKENS
      .saturating_add(bob_pool_after_session_2 * expected_max_rewards);

    assert_eq!(
      TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      expected_alice_staked_principal_after_session_2
    );

    assert_eq!(
      TidefiStaking::account_stakes(BOB_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      expected_bob_staked_principal_after_session_2
    );

    // 2 empty sessions
    assert_ok!(TidefiStaking::on_session_end(
      3,
      Vec::new(),
      fees_pallet_account
    ));
    assert_ok!(TidefiStaking::on_session_end(
      4,
      Vec::new(),
      fees_pallet_account
    ));

    run_on_idle_hook::<Test>();

    // balance should be the same
    assert_eq!(
      TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      expected_alice_staked_principal_after_session_2
    );

    assert_eq!(
      TidefiStaking::account_stakes(BOB_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      expected_bob_staked_principal_after_session_2
    );

    assert_ok!(TidefiStaking::stake(
      RuntimeOrigin::signed(CHARLIE_ACCOUNT_ID),
      TEST_TOKEN_CURRENCY_ID,
      CHARLIE_STAKE_FOUR_HUNDRED_TEST_TOKENS,
      FIFTEEN_DAYS
    ));

    assert_ok!(TidefiStaking::on_session_end(
      5,
      vec![(
        TEST_TOKEN_CURRENCY_ID,
        SESSION_TOTAL_FEES_ONE_HUNDRED_TEST_TOKENS
      )],
      fees_pallet_account
    ));

    let staking_pool_before_compound_session_5 =
      TidefiStaking::staking_pool(TEST_TOKEN_CURRENCY_ID).unwrap_or(0);
    run_on_idle_hook::<Test>();

    // charlie is processed first
    let charlie_pool_percentage_after_session_5 = Perquintill::from_rational(
      CHARLIE_STAKE_FOUR_HUNDRED_TEST_TOKENS,
      staking_pool_before_compound_session_5,
    );

    let expected_charlie_staked_principal_after_session_5 = CHARLIE_STAKE_FOUR_HUNDRED_TEST_TOKENS
      .saturating_add(charlie_pool_percentage_after_session_5 * expected_max_rewards);

    assert_eq!(
      TidefiStaking::account_stakes(CHARLIE_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      expected_charlie_staked_principal_after_session_5
    );

    let alice_pool_percentage_after_session_5 = Perquintill::from_rational(
      expected_alice_staked_principal_after_session_2,
      staking_pool_before_compound_session_5.saturating_add(
        // get only the rewards from charlie to add them to the staking pool variable
        expected_charlie_staked_principal_after_session_5
          .saturating_sub(CHARLIE_STAKE_FOUR_HUNDRED_TEST_TOKENS),
      ),
    );

    let expected_alice_staked_principal_after_session_5 =
      expected_alice_staked_principal_after_session_2
        .saturating_add(alice_pool_percentage_after_session_5 * expected_max_rewards);

    assert_eq!(
      TidefiStaking::account_stakes(ALICE_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      expected_alice_staked_principal_after_session_5
    );

    let bob_pool_percentage_after_session_5 = Perquintill::from_rational(
      expected_bob_staked_principal_after_session_2,
      staking_pool_before_compound_session_5
        .saturating_add(
          // get only the rewards from charlie to add them to the staking pool variable
          // as its a new stake
          expected_charlie_staked_principal_after_session_5
            .saturating_sub(CHARLIE_STAKE_FOUR_HUNDRED_TEST_TOKENS),
        )
        // add alice rewards
        .saturating_add(alice_pool_percentage_after_session_5 * expected_max_rewards),
    );

    let expected_bob_staked_principal_after_session_5 =
      expected_bob_staked_principal_after_session_2
        .saturating_add(bob_pool_percentage_after_session_5 * expected_max_rewards);

    assert_eq!(
      TidefiStaking::account_stakes(BOB_ACCOUNT_ID)
        .first()
        .unwrap()
        .principal,
      expected_bob_staked_principal_after_session_5
    );

    // make sure we can drain all sessions
    assert_eq!(PendingStoredSessions::<Test>::count(), 1);
    assert_eq!(SessionTotalFees::<Test>::iter().count(), 1);
    run_on_idle_hook::<Test>();
    assert_eq!(PendingStoredSessions::<Test>::count(), 0);
    assert_eq!(SessionTotalFees::<Test>::iter().count(), 0);
  });
}

#[test]
fn test_migration_v2() {
  new_test_ext().execute_with(|| {
    crate::migrations::v2::migrate::<Test>();
    crate::migrations::v2::post_migration::<Test>();
  });
}
