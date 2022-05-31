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
    new_test_ext, Adapter, Assets, Balances, Event as MockEvent, Oracle, Origin, Quorum,
    SwapLimitByAccount, System, Test, Tidefi,
  },
  pallet::*,
};
use frame_support::{
  assert_err, assert_noop, assert_ok,
  traits::fungibles::{Inspect, Mutate},
  BoundedVec,
};
use pallet_assets::{Account, Error as AssetsError};
use pallet_balances::Error as BalancesError;
use pallet_oracle::{AccountSwaps, Error as OracleError};
use sp_runtime::{traits::BadOrigin, Permill};
use std::str::FromStr;
use tidefi_primitives::{
  pallet::OracleExt, Balance, CurrencyId, Hash, ProposalType, SwapStatus, SwapType, Withdrawal,
};

type AccountId = u64;

const ALICE_ACCOUNT_ID: AccountId = 1;
const BOB_ACCOUNT_ID: AccountId = 2;

const ONE_TDFY: u128 = 1_000_000_000_000;

const TEMP_ASSET_ID: u32 = 4;
const TEMP_CURRENCY_ID: CurrencyId = CurrencyId::Wrapped(TEMP_ASSET_ID);
const TEMP_ASSET_IS_SUFFICIENT: bool = true;
const TEMP_ASSET_MIN_BALANCE: u128 = 1;
const ONE_TEMP: u128 = 100;

// TEMP Asset Metadata
const TEMP_ASSET_NAME: &str = "TEMP";
const TEMP_ASSET_SYMBOL: &str = "TEMP";
const TEMP_ASSET_NUMBER_OF_DECIMAL_PLACES: u8 = 8;

const BLOCK_NUMBER_ZERO: u64 = 0;

struct Context {
  sender: AccountId,
  receiver: AccountId,
  test_assets: Vec<CurrencyId>,
  amount: u128,
  external_address: Vec<u8>,
  request_id: Hash,
  proposal_id: Hash,
}

impl Default for Context {
  fn default() -> Self {
    Context {
      sender: ALICE_ACCOUNT_ID,
      receiver: BOB_ACCOUNT_ID,
      test_assets: vec![CurrencyId::Tdfy, TEMP_CURRENCY_ID],
      amount: 10,
      external_address: vec![0; 32],
      request_id: Hash::from_str(
        "0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e",
      )
      .unwrap_or_default(),
      proposal_id: Hash::from_str(
        "0x02a204a25c36f8c88eea76e08cdaa22a0569ef630bf4416db72abb9fb2445f2b",
      )
      .unwrap(),
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

  fn mint_tdfy(self, account: AccountId, amount: u128) -> Self {
    Self::mint_asset_for_accounts(vec![account], CurrencyId::Tdfy, amount);
    assert_eq!(Adapter::balance(CurrencyId::Tdfy, &account), amount);
    self
  }

  fn mint_temp(self, account: AccountId, amount: u128) -> Self {
    Self::mint_asset_for_accounts(vec![account], TEMP_CURRENCY_ID, amount);
    assert_eq!(Adapter::balance(TEMP_CURRENCY_ID, &account), amount);
    self
  }

  fn mint_asset_for_accounts(accounts: Vec<AccountId>, asset: CurrencyId, amount: u128) {
    for account in accounts {
      assert_ok!(Adapter::mint_into(asset, &account, amount));
    }
  }

  fn create_temp_asset_and_metadata(self) -> Self {
    let temp_asset_owner = ALICE_ACCOUNT_ID;

    assert_ok!(Assets::force_create(
      Origin::root(),
      TEMP_ASSET_ID,
      temp_asset_owner,
      TEMP_ASSET_IS_SUFFICIENT,
      TEMP_ASSET_MIN_BALANCE
    ));

    assert_ok!(Assets::set_metadata(
      Origin::signed(temp_asset_owner),
      TEMP_ASSET_ID,
      TEMP_ASSET_NAME.into(),
      TEMP_ASSET_SYMBOL.into(),
      TEMP_ASSET_NUMBER_OF_DECIMAL_PLACES
    ));

    self
  }

  fn add_tdfy_to_temp_limit_swap(self, account_id: AccountId, tdfys: u128, temps: u128) -> Self {
    Oracle::add_new_swap_in_queue(
      account_id,
      CurrencyId::Tdfy,
      tdfys,
      TEMP_CURRENCY_ID,
      temps,
      BLOCK_NUMBER_ZERO,
      [0u8; 32],
      false,
      SwapType::Limit,
      Permill::from_parts(1),
    )
    .unwrap();

    assert_eq!(
      Oracle::account_swaps(BOB_ACCOUNT_ID)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == self.request_id),
      Some(&(self.request_id, SwapStatus::Pending))
    );
    self
  }

  fn add_temp_to_tdfy_limit_swap(self, account_id: AccountId, temps: u128, tdfys: u128) -> Self {
    Oracle::add_new_swap_in_queue(
      account_id,
      TEMP_CURRENCY_ID,
      temps,
      CurrencyId::Tdfy,
      tdfys,
      BLOCK_NUMBER_ZERO,
      [0u8; 32],
      false,
      SwapType::Limit,
      Permill::from_parts(1),
    )
    .unwrap();

    assert_eq!(
      Oracle::account_swaps(BOB_ACCOUNT_ID)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == self.request_id),
      Some(&(self.request_id, SwapStatus::Pending))
    );
    self
  }
}

fn get_alice_balance(currency_id: CurrencyId) -> Balance {
  get_account_balance(ALICE_ACCOUNT_ID, currency_id)
}

fn get_bob_balance(currency_id: CurrencyId) -> Balance {
  get_account_balance(BOB_ACCOUNT_ID, currency_id)
}

fn get_account_balance(account_id: AccountId, currency_id: CurrencyId) -> Balance {
  match currency_id {
    CurrencyId::Tdfy => Balances::free_balance(account_id),
    CurrencyId::Wrapped(_) => Adapter::balance(currency_id, &account_id),
  }
}

fn get_account_reserved(account_id: AccountId, currency_id: CurrencyId) -> Balance {
  match currency_id {
    CurrencyId::Tdfy => Balances::reserved_balance(account_id),
    CurrencyId::Wrapped(asset_id) => Account::<Test>::get(account_id, asset_id).unwrap().reserved,
  }
}

fn assert_withdrawal_proposal_exists_in_storage(context: &Context) {
  assert_eq!(
    Quorum::proposals().into_inner().first().unwrap(),
    &(
      context.proposal_id,
      BLOCK_NUMBER_ZERO,
      ProposalType::Withdrawal(Withdrawal {
        account_id: context.sender,
        asset_id: TEMP_CURRENCY_ID,
        amount: context.amount,
        external_address: BoundedVec::try_from(context.external_address.clone()).unwrap(),
        block_number: BLOCK_NUMBER_ZERO,
      })
    )
  );
}

fn assert_cancelled_swap_is_set_to_none(context: &Context) {
  assert!(Oracle::swaps(context.request_id).is_none());
}

fn assert_cancelled_swap_is_deleted_from_account_swaps(context: &Context) {
  assert_eq!(
    Oracle::account_swaps(BOB_ACCOUNT_ID)
      .unwrap()
      .iter()
      .find(|(request_id, _)| *request_id == context.request_id),
    None
  );
}

fn assert_event_is_emitted_transfer(context: &Context, currency_id: CurrencyId) {
  System::assert_has_event(MockEvent::Tidefi(Event::Transfer {
    from_account_id: context.sender,
    to_account_id: context.receiver,
    currency_id: currency_id,
    amount: context.amount,
  }));
}

fn assert_event_is_emitted_withdrawal(context: &Context, currency_id: CurrencyId) {
  System::assert_has_event(MockEvent::Tidefi(Event::Withdrawal {
    account: context.sender,
    currency_id: currency_id,
    amount: context.amount,
    external_address: context.external_address.clone(),
  }));
}

fn assert_event_is_emitted_swap_cancelled(context: &Context) {
  System::assert_has_event(MockEvent::Tidefi(Event::SwapCancelled {
    request_id: context.request_id,
  }));
}

mod transfer {
  use super::*;

  #[test]
  fn succeeds() {
    new_test_ext().execute_with(|| {
      let context = Context::default()
        .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
        .mint_tdfy(BOB_ACCOUNT_ID, 10 * ONE_TDFY)
        .create_temp_asset_and_metadata()
        .mint_temp(ALICE_ACCOUNT_ID, 10 * ONE_TEMP);

      for currency_id in context.test_assets.clone() {
        let alice_balance_before = get_alice_balance(currency_id);
        let bob_balance_before = get_bob_balance(currency_id);

        assert_ok!(Tidefi::transfer(
          Origin::signed(context.sender),
          context.receiver,
          currency_id,
          context.amount
        ));

        assert_eq!(
          alice_balance_before - context.amount,
          get_alice_balance(currency_id)
        );
        assert_eq!(
          bob_balance_before + context.amount,
          get_bob_balance(currency_id)
        );
        assert_event_is_emitted_transfer(&context, currency_id);
      }
    });
  }

  mod fails_when {
    use super::*;

    #[test]
    fn not_signed() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, 10 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(ALICE_ACCOUNT_ID, 10 * ONE_TEMP);

        for currency_id in context.test_assets.clone() {
          assert_noop!(
            Tidefi::transfer(
              Origin::none(),
              context.receiver,
              currency_id,
              context.amount
            ),
            BadOrigin
          );
        }
      });
    }

    #[test]
    fn non_tdfy_asset_is_disabled() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, 10 * ONE_TDFY);

        assert_noop!(
          Tidefi::transfer(
            Origin::signed(context.sender),
            context.receiver,
            TEMP_CURRENCY_ID,
            context.amount
          ),
          Error::<Test>::AssetDisabled
        );
      });
    }

    #[test]
    fn non_tdfy_fund_sender_does_not_exist() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, 10 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(ALICE_ACCOUNT_ID, 10 * ONE_TEMP);

        let invalid_sender: AccountId = 10;
        assert_noop!(
          Tidefi::transfer(
            Origin::signed(invalid_sender),
            context.receiver,
            TEMP_CURRENCY_ID,
            context.amount
          ),
          AssetsError::<Test>::NoAccount
        );
      });
    }

    #[test]
    fn sender_has_insufficient_tdfy_funds_to_send() {
      new_test_ext().execute_with(|| {
        let context = Context::default().mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY);

        assert_noop!(
          Tidefi::transfer(
            Origin::signed(context.sender),
            context.receiver,
            CurrencyId::Tdfy,
            10 * ONE_TDFY + 1
          ),
          BalancesError::<Test>::InsufficientBalance
        );
      });
    }

    #[test]
    fn sender_has_insufficient_non_tdfy_funds_to_send() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(ALICE_ACCOUNT_ID, 10 * ONE_TEMP);

        assert_noop!(
          Tidefi::transfer(
            Origin::signed(context.sender),
            context.receiver,
            TEMP_CURRENCY_ID,
            10 * ONE_TEMP + 1
          ),
          AssetsError::<Test>::BalanceLow
        );
      });
    }

    #[test]
    fn sender_has_insufficient_tdfy_left_to_keep_alive() {
      new_test_ext().execute_with(|| {
        let context = Context::default().mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY);

        assert_noop!(
          Tidefi::transfer(
            Origin::signed(context.sender),
            context.receiver,
            CurrencyId::Tdfy,
            10 * ONE_TDFY
          ),
          BalancesError::<Test>::KeepAlive
        );
      });
    }

    #[test]
    fn receiver_has_not_enough_tdfy_to_exist() {
      new_test_ext().execute_with(|| {
        let context = Context::default().mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY);

        assert_noop!(
          Tidefi::transfer(
            Origin::signed(context.sender),
            context.receiver,
            CurrencyId::Tdfy,
            context.amount
          ),
          BalancesError::<Test>::ExistentialDeposit
        );
      });
    }
  }
}

mod withdrawal {
  use super::*;

  #[test]
  fn succeeds() {
    new_test_ext().execute_with(|| {
      let context = Context::default()
        .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
        .create_temp_asset_and_metadata()
        .mint_temp(ALICE_ACCOUNT_ID, 10 * ONE_TEMP);

      let alice_balance_before = get_alice_balance(TEMP_CURRENCY_ID);

      assert_ok!(Tidefi::withdrawal(
        Origin::signed(context.sender),
        TEMP_CURRENCY_ID,
        context.amount,
        context.external_address.clone(),
      ));

      assert_eq!(alice_balance_before, get_alice_balance(TEMP_CURRENCY_ID));
      assert_withdrawal_proposal_exists_in_storage(&context);
      assert_event_is_emitted_withdrawal(&context, TEMP_CURRENCY_ID);
    });
  }

  mod fails_when {
    use super::*;

    #[test]
    fn not_signed() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(ALICE_ACCOUNT_ID, 10 * ONE_TEMP);

        assert_noop!(
          Tidefi::withdrawal(
            Origin::none(),
            TEMP_CURRENCY_ID,
            context.amount,
            context.external_address.clone(),
          ),
          BadOrigin
        );
      });
    }

    #[ignore]
    #[test]
    fn quorum_is_paused() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(ALICE_ACCOUNT_ID, 10 * ONE_TEMP)
          .set_oracle_status(false);

        assert_noop!(
          Tidefi::withdrawal(
            Origin::signed(context.sender),
            TEMP_CURRENCY_ID,
            context.amount,
            context.external_address.clone(),
          ),
          Error::<Test>::QuorumPaused
        );
      });
    }

    #[test]
    fn asset_is_disabled() {
      new_test_ext().execute_with(|| {
        let context = Context::default().mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY);

        assert_noop!(
          Tidefi::withdrawal(
            Origin::signed(context.sender),
            TEMP_CURRENCY_ID,
            context.amount,
            context.external_address.clone(),
          ),
          Error::<Test>::AssetDisabled
        );
      });
    }

    #[test]
    fn asset_is_tdfy() {
      new_test_ext().execute_with(|| {
        let context = Context::default().mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY);

        assert_noop!(
          Tidefi::withdrawal(
            Origin::signed(context.sender),
            CurrencyId::Tdfy,
            context.amount,
            context.external_address.clone(),
          ),
          Error::<Test>::CannotWithdrawTdfy
        );
      });
    }

    #[test]
    fn amount_is_greater_than_sender_balance() {
      new_test_ext().execute_with(|| {
        let total_temp_supply = 20 * ONE_TEMP;
        let alice_temp_balance = 10 * ONE_TEMP;
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(ALICE_ACCOUNT_ID, alice_temp_balance)
          .mint_temp(BOB_ACCOUNT_ID, total_temp_supply - alice_temp_balance);

        assert_noop!(
          Tidefi::withdrawal(
            Origin::signed(context.sender),
            TEMP_CURRENCY_ID,
            alice_temp_balance + 1,
            context.external_address.clone(),
          ),
          Error::<Test>::WithdrawAmountGreaterThanAccountBalance
        );
      });
    }

    #[test]
    fn amount_is_greater_than_asset_supply() {
      new_test_ext().execute_with(|| {
        let initial_temp_amount = 10 * ONE_TEMP;
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(ALICE_ACCOUNT_ID, initial_temp_amount);

        assert_noop!(
          Tidefi::withdrawal(
            Origin::signed(context.sender),
            TEMP_CURRENCY_ID,
            initial_temp_amount + 1,
            context.external_address.clone(),
          ),
          Error::<Test>::WithdrawAmountGreaterThanAssetSupply
        );
      });
    }

    #[test]
    fn account_asset_is_frozen() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(ALICE_ACCOUNT_ID, 10 * ONE_TEMP);

        assert_ok!(Assets::freeze(
          Origin::signed(context.sender),
          TEMP_ASSET_ID,
          ALICE_ACCOUNT_ID
        ));

        assert_noop!(
          Tidefi::withdrawal(
            Origin::signed(context.sender),
            TEMP_CURRENCY_ID,
            10 * ONE_TEMP,
            context.external_address.clone(),
          ),
          Error::<Test>::AccountAssetFrozen
        );
      });
    }

    #[test]
    fn sender_balance_would_be_reduced_to_zero() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(ALICE_ACCOUNT_ID, 10 * ONE_TEMP);

        assert_noop!(
          Tidefi::withdrawal(
            Origin::signed(context.sender),
            TEMP_CURRENCY_ID,
            10 * ONE_TEMP,
            context.external_address.clone(),
          ),
          Error::<Test>::ReducedToZero
        );
      });
    }
  }
}

mod swap {
  use super::*;

  #[test]
  fn succeeds() {
    new_test_ext().execute_with(|| {
      let context = Context::default()
        .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
        .mint_tdfy(BOB_ACCOUNT_ID, 20 * ONE_TDFY)
        .create_temp_asset_and_metadata()
        .mint_temp(BOB_ACCOUNT_ID, 10_000 * ONE_TEMP);

      // Submit request
      assert_ok!(Tidefi::swap(
        Origin::signed(BOB_ACCOUNT_ID),
        CurrencyId::Tdfy,
        10 * ONE_TDFY,
        TEMP_CURRENCY_ID,
        200 * ONE_TEMP,
        SwapType::Limit,
        None
      ));

      // swap confirmation for bob (user)
      System::assert_has_event(MockEvent::Tidefi(Event::Swap {
        request_id: context.request_id,
        account: BOB_ACCOUNT_ID,
        currency_id_from: CurrencyId::Tdfy,
        amount_from: 10 * ONE_TDFY,
        currency_id_to: TEMP_CURRENCY_ID,
        amount_to: 200 * ONE_TEMP,
        extrinsic_hash: [
          14, 87, 81, 192, 38, 229, 67, 178, 232, 171, 46, 176, 96, 153, 218, 161, 209, 229, 223,
          71, 119, 143, 119, 135, 250, 171, 69, 205, 241, 47, 227, 168,
        ],
        slippage_tolerance: Permill::from_parts(1),
        swap_type: SwapType::Limit,
        is_market_maker: false,
      }));
    })
  }

  mod fails_when {
    use super::*;

    #[test]
    fn not_signed() {
      new_test_ext().execute_with(|| {
        Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, 20 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(BOB_ACCOUNT_ID, 10_000 * ONE_TEMP);

        assert_noop!(
          Tidefi::swap(
            Origin::none(),
            CurrencyId::Tdfy,
            10 * ONE_TDFY,
            TEMP_CURRENCY_ID,
            200 * ONE_TEMP,
            SwapType::Limit,
            None
          ),
          BadOrigin
        );
      });
    }

    #[test]
    fn oracle_is_paused() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, 20 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(BOB_ACCOUNT_ID, 10_000 * ONE_TEMP)
          .set_oracle_status(false);

        assert_noop!(
          Tidefi::swap(
            Origin::signed(context.sender),
            CurrencyId::Tdfy,
            10 * ONE_TDFY,
            TEMP_CURRENCY_ID,
            200 * ONE_TEMP,
            SwapType::Limit,
            None
          ),
          Error::<Test>::OraclePaused
        );
      });
    }

    #[test]
    fn asset_is_disabled() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, 20 * ONE_TDFY);

        assert_noop!(
          Tidefi::swap(
            Origin::signed(context.sender),
            TEMP_CURRENCY_ID,
            200 * ONE_TEMP,
            CurrencyId::Tdfy,
            10 * ONE_TDFY,
            SwapType::Limit,
            None
          ),
          Error::<Test>::AssetDisabled
        );

        assert_noop!(
          Tidefi::swap(
            Origin::signed(context.sender),
            CurrencyId::Tdfy,
            10 * ONE_TDFY,
            TEMP_CURRENCY_ID,
            200 * ONE_TEMP,
            SwapType::Limit,
            None
          ),
          Error::<Test>::AssetDisabled
        );
      });
    }

    #[test]
    fn account_swap_overflow() {
      new_test_ext().execute_with(|| {
        Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, 20 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(BOB_ACCOUNT_ID, 10_000 * ONE_TEMP);

        AccountSwaps::<Test>::insert(
          BOB_ACCOUNT_ID,
          BoundedVec::try_from(vec![
            (Hash::zero(), SwapStatus::Pending);
            usize::try_from(SwapLimitByAccount::get()).unwrap()
          ])
          .unwrap(),
        );

        assert_err!(
          Tidefi::swap(
            Origin::signed(BOB_ACCOUNT_ID),
            CurrencyId::Tdfy,
            10 * ONE_TDFY,
            TEMP_CURRENCY_ID,
            200 * ONE_TEMP,
            SwapType::Limit,
            None
          ),
          OracleError::<Test>::SwapOverflow
        );
      });
    }

    #[test]
    fn sell_asset_amount_is_greater_than_requester_balance() {
      new_test_ext().execute_with(|| {
        let total_temp_supply = 20 * ONE_TEMP;
        let bob_temp_balance = 10 * ONE_TEMP;
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(ALICE_ACCOUNT_ID, total_temp_supply - bob_temp_balance)
          .mint_temp(BOB_ACCOUNT_ID, bob_temp_balance);

        assert_noop!(
          Tidefi::swap(
            Origin::signed(context.sender),
            TEMP_CURRENCY_ID,
            bob_temp_balance + 1,
            CurrencyId::Tdfy,
            ONE_TDFY,
            SwapType::Limit,
            None
          ),
          Error::<Test>::WithdrawAmountGreaterThanAccountBalance
        );
      });
    }

    #[test]
    fn sell_asset_amount_is_greater_than_asset_supply() {
      new_test_ext().execute_with(|| {
        let initial_temp_amount = 10 * ONE_TEMP;
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(ALICE_ACCOUNT_ID, initial_temp_amount);

        assert_noop!(
          Tidefi::swap(
            Origin::signed(context.sender),
            TEMP_CURRENCY_ID,
            initial_temp_amount + 1,
            CurrencyId::Tdfy,
            ONE_TDFY,
            SwapType::Limit,
            None
          ),
          Error::<Test>::WithdrawAmountGreaterThanAssetSupply
        );
      });
    }

    #[test]
    fn account_asset_is_frozen() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(ALICE_ACCOUNT_ID, 10 * ONE_TEMP);

        assert_ok!(Assets::freeze(
          Origin::signed(context.sender),
          TEMP_ASSET_ID,
          ALICE_ACCOUNT_ID
        ));

        assert_noop!(
          Tidefi::swap(
            Origin::signed(context.sender),
            TEMP_CURRENCY_ID,
            10 * ONE_TEMP,
            CurrencyId::Tdfy,
            ONE_TDFY,
            SwapType::Limit,
            None
          ),
          Error::<Test>::AccountAssetFrozen
        );
      });
    }

    #[test]
    fn sender_balance_would_be_reduced_to_zero() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, 10 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(ALICE_ACCOUNT_ID, 10 * ONE_TEMP);

        assert_noop!(
          Tidefi::swap(
            Origin::signed(context.sender),
            TEMP_CURRENCY_ID,
            10 * ONE_TEMP,
            CurrencyId::Tdfy,
            ONE_TDFY,
            SwapType::Limit,
            None
          ),
          Error::<Test>::ReducedToZero
        );
      });
    }
  }
}

mod cancel_swap {
  use super::*;

  mod succeeds {
    use super::*;

    mod by_requester {
      use super::*;

      #[test]
      fn from_tdfy() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
            .mint_tdfy(BOB_ACCOUNT_ID, 20 * ONE_TDFY)
            .create_temp_asset_and_metadata()
            .mint_temp(BOB_ACCOUNT_ID, 10_000 * ONE_TEMP)
            .add_tdfy_to_temp_limit_swap(BOB_ACCOUNT_ID, 10 * ONE_TDFY, 200 * ONE_TEMP);

          let requester_balance_before = get_account_balance(BOB_ACCOUNT_ID, CurrencyId::Tdfy);
          let requester_reserved = get_account_reserved(BOB_ACCOUNT_ID, CurrencyId::Tdfy);

          assert_ok!(Tidefi::cancel_swap(
            Origin::signed(BOB_ACCOUNT_ID),
            context.request_id,
          ));

          assert_eq!(
            Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
            requester_balance_before + requester_reserved
          );

          assert_cancelled_swap_is_set_to_none(&context);
          assert_cancelled_swap_is_deleted_from_account_swaps(&context);
          assert_event_is_emitted_swap_cancelled(&context);
        })
      }

      #[test]
      fn from_non_tdfy() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
            .mint_tdfy(BOB_ACCOUNT_ID, 20 * ONE_TDFY)
            .create_temp_asset_and_metadata()
            .mint_temp(BOB_ACCOUNT_ID, 10_000 * ONE_TEMP)
            .add_temp_to_tdfy_limit_swap(BOB_ACCOUNT_ID, 200 * ONE_TEMP, 10 * ONE_TDFY);

          let requester_balance_before = get_account_balance(BOB_ACCOUNT_ID, TEMP_CURRENCY_ID);
          let requester_reserved = get_account_reserved(BOB_ACCOUNT_ID, TEMP_CURRENCY_ID);

          assert_ok!(Tidefi::cancel_swap(
            Origin::signed(ALICE_ACCOUNT_ID),
            context.request_id,
          ));

          assert_eq!(
            Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
            requester_balance_before + requester_reserved
          );

          assert_cancelled_swap_is_set_to_none(&context);
          assert_cancelled_swap_is_deleted_from_account_swaps(&context);
          assert_event_is_emitted_swap_cancelled(&context);
        })
      }
    }

    mod by_tidefy_owner {
      use super::*;

      #[test]
      fn from_tdfy() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
            .mint_tdfy(BOB_ACCOUNT_ID, 20 * ONE_TDFY)
            .create_temp_asset_and_metadata()
            .mint_temp(BOB_ACCOUNT_ID, 10_000 * ONE_TEMP)
            .add_tdfy_to_temp_limit_swap(BOB_ACCOUNT_ID, 10 * ONE_TDFY, 200 * ONE_TEMP);

          let requester_balance_before = get_account_balance(BOB_ACCOUNT_ID, CurrencyId::Tdfy);
          let requester_reserved = get_account_reserved(BOB_ACCOUNT_ID, CurrencyId::Tdfy);

          assert_ok!(Tidefi::cancel_swap(
            Origin::signed(ALICE_ACCOUNT_ID),
            context.request_id,
          ));

          assert_eq!(
            Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
            requester_balance_before + requester_reserved
          );

          assert_cancelled_swap_is_set_to_none(&context);
          assert_cancelled_swap_is_deleted_from_account_swaps(&context);
          assert_event_is_emitted_swap_cancelled(&context);
        })
      }

      #[test]
      fn from_non_tdfy() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
            .mint_tdfy(BOB_ACCOUNT_ID, 20 * ONE_TDFY)
            .create_temp_asset_and_metadata()
            .mint_temp(BOB_ACCOUNT_ID, 10_000 * ONE_TEMP)
            .add_tdfy_to_temp_limit_swap(BOB_ACCOUNT_ID, 10 * ONE_TDFY, 200 * ONE_TEMP);

          let requester_balance_before = get_account_balance(BOB_ACCOUNT_ID, TEMP_CURRENCY_ID);
          let requester_reserved = get_account_reserved(BOB_ACCOUNT_ID, TEMP_CURRENCY_ID);

          assert_ok!(Tidefi::cancel_swap(
            Origin::signed(ALICE_ACCOUNT_ID),
            context.request_id,
          ));

          assert_eq!(
            Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
            requester_balance_before + requester_reserved
          );

          assert_cancelled_swap_is_set_to_none(&context);
          assert_cancelled_swap_is_deleted_from_account_swaps(&context);
          assert_event_is_emitted_swap_cancelled(&context);
        })
      }
    }
  }

  mod fails_when {
    use super::*;

    #[test]
    fn not_signed() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, 20 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(BOB_ACCOUNT_ID, 10_000 * ONE_TEMP)
          .add_tdfy_to_temp_limit_swap(BOB_ACCOUNT_ID, 10 * ONE_TDFY, 200 * ONE_TEMP);

        assert_noop!(
          Tidefi::cancel_swap(Origin::none(), context.request_id),
          BadOrigin
        );
      })
    }

    #[test]
    fn oracle_is_paused() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, 20 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(BOB_ACCOUNT_ID, 10_000 * ONE_TEMP)
          .add_tdfy_to_temp_limit_swap(BOB_ACCOUNT_ID, 10 * ONE_TDFY, 200 * ONE_TEMP)
          .set_oracle_status(false);

        assert_noop!(
          Tidefi::cancel_swap(Origin::signed(BOB_ACCOUNT_ID), context.request_id),
          Error::<Test>::OraclePaused
        );

        assert_noop!(
          Tidefi::cancel_swap(Origin::signed(ALICE_ACCOUNT_ID), context.request_id),
          Error::<Test>::OraclePaused
        );
      });
    }

    #[test]
    fn request_id_is_invalid() {
      new_test_ext().execute_with(|| {
        Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, 20 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(BOB_ACCOUNT_ID, 10_000 * ONE_TEMP)
          .add_tdfy_to_temp_limit_swap(BOB_ACCOUNT_ID, 10 * ONE_TDFY, 200 * ONE_TEMP);

        assert_noop!(
          Tidefi::cancel_swap(Origin::signed(BOB_ACCOUNT_ID), Hash::zero()),
          OracleError::<Test>::InvalidRequestId
        );

        assert_noop!(
          Tidefi::cancel_swap(Origin::signed(ALICE_ACCOUNT_ID), Hash::zero()),
          OracleError::<Test>::InvalidRequestId
        );
      })
    }

    #[test]
    fn sender_is_not_requester_or_tidefi_pallet_owner() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, 20 * ONE_TDFY)
          .create_temp_asset_and_metadata()
          .mint_temp(BOB_ACCOUNT_ID, 10_000 * ONE_TEMP)
          .add_tdfy_to_temp_limit_swap(BOB_ACCOUNT_ID, 10 * ONE_TDFY, 200 * ONE_TEMP);

        const UNKNOWN_ACCOUNT: u64 = 100;
        assert_noop!(
          Tidefi::cancel_swap(Origin::signed(UNKNOWN_ACCOUNT), context.request_id),
          OracleError::<Test>::AccessDenied
        );
      });
    }

    mod release_fails {
      use super::*;

      #[test]
      fn as_account_asset_does_not_exist() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
            .mint_tdfy(BOB_ACCOUNT_ID, 20 * ONE_TDFY)
            .create_temp_asset_and_metadata()
            .mint_temp(BOB_ACCOUNT_ID, 10_000 * ONE_TEMP)
            .add_temp_to_tdfy_limit_swap(BOB_ACCOUNT_ID, 200 * ONE_TEMP, 10 * ONE_TDFY);

          Account::<Test>::remove(&BOB_ACCOUNT_ID, TEMP_ASSET_ID);

          assert_noop!(
            Tidefi::cancel_swap(Origin::signed(BOB_ACCOUNT_ID), context.request_id),
            OracleError::<Test>::ReleaseFailed
          );

          assert_noop!(
            Tidefi::cancel_swap(Origin::signed(ALICE_ACCOUNT_ID), context.request_id),
            OracleError::<Test>::ReleaseFailed
          );
        });
      }

      #[test]
      fn as_release_amount_is_greater_than_reserved_amount() {
        new_test_ext().execute_with(|| {
          let context = Context::default()
            .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
            .mint_tdfy(BOB_ACCOUNT_ID, 20 * ONE_TDFY)
            .create_temp_asset_and_metadata()
            .mint_temp(BOB_ACCOUNT_ID, 10_000 * ONE_TEMP)
            .add_temp_to_tdfy_limit_swap(BOB_ACCOUNT_ID, 200 * ONE_TEMP, 10 * ONE_TDFY);

          Account::<Test>::try_mutate(BOB_ACCOUNT_ID, TEMP_ASSET_ID, |maybe_account| {
            maybe_account
              .as_mut()
              .ok_or(())
              .map(|account| account.reserved = account.reserved - 1)
          })
          .unwrap();

          assert_noop!(
            Tidefi::cancel_swap(Origin::signed(BOB_ACCOUNT_ID), context.request_id),
            OracleError::<Test>::ReleaseFailed
          );

          assert_noop!(
            Tidefi::cancel_swap(Origin::signed(ALICE_ACCOUNT_ID), context.request_id),
            OracleError::<Test>::ReleaseFailed
          );
        });
      }
    }
  }
}
