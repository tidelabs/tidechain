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

// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Implementations for fungibles trait.

use super::*;

impl<T: Config<I>, I: 'static> fungibles::Inspect<<T as SystemConfig>::AccountId> for Pallet<T, I> {
  type AssetId = T::AssetId;
  type Balance = T::Balance;

  fn total_issuance(asset: Self::AssetId) -> Self::Balance {
    Asset::<T, I>::get(asset)
      .map(|x| x.supply)
      .unwrap_or_else(Zero::zero)
  }

  fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
    Asset::<T, I>::get(asset)
      .map(|x| x.min_balance)
      .unwrap_or_else(Zero::zero)
  }

  fn balance(asset: Self::AssetId, who: &<T as SystemConfig>::AccountId) -> Self::Balance {
    Pallet::<T, I>::balance(asset, who)
  }

  fn reducible_balance(
    asset: Self::AssetId,
    who: &<T as SystemConfig>::AccountId,
    keep_alive: bool,
  ) -> Self::Balance {
    Pallet::<T, I>::reducible_balance(asset, who, keep_alive).unwrap_or(Zero::zero())
  }

  fn can_deposit(
    asset: Self::AssetId,
    who: &<T as SystemConfig>::AccountId,
    amount: Self::Balance,
    mint: bool,
  ) -> DepositConsequence {
    Pallet::<T, I>::can_increase(asset, who, amount, mint)
  }

  fn can_withdraw(
    asset: Self::AssetId,
    who: &<T as SystemConfig>::AccountId,
    amount: Self::Balance,
  ) -> WithdrawConsequence<Self::Balance> {
    Pallet::<T, I>::can_decrease(asset, who, amount, false)
  }
}

impl<T: Config<I>, I: 'static> fungibles::InspectMetadata<<T as SystemConfig>::AccountId>
  for Pallet<T, I>
{
  /// Return the name of an asset.
  fn name(asset: &Self::AssetId) -> Vec<u8> {
    Metadata::<T, I>::get(asset).name.to_vec()
  }

  /// Return the symbol of an asset.
  fn symbol(asset: &Self::AssetId) -> Vec<u8> {
    Metadata::<T, I>::get(asset).symbol.to_vec()
  }

  /// Return the decimals of an asset.
  fn decimals(asset: &Self::AssetId) -> u8 {
    Metadata::<T, I>::get(asset).decimals
  }
}

impl<T: Config<I>, I: 'static> fungibles::Mutate<<T as SystemConfig>::AccountId> for Pallet<T, I> {
  fn mint_into(
    asset: Self::AssetId,
    who: &<T as SystemConfig>::AccountId,
    amount: Self::Balance,
  ) -> DispatchResult {
    Self::do_mint(asset, who, amount, None)
  }

  fn burn_from(
    asset: Self::AssetId,
    who: &<T as SystemConfig>::AccountId,
    amount: Self::Balance,
  ) -> Result<Self::Balance, DispatchError> {
    let f = DebitFlags {
      keep_alive: false,
      best_effort: false,
    };
    Self::do_burn(asset, who, amount, None, f)
  }

  fn slash(
    asset: Self::AssetId,
    who: &<T as SystemConfig>::AccountId,
    amount: Self::Balance,
  ) -> Result<Self::Balance, DispatchError> {
    let f = DebitFlags {
      keep_alive: false,
      best_effort: true,
    };
    Self::do_burn(asset, who, amount, None, f)
  }
}

impl<T: Config<I>, I: 'static> fungibles::Transfer<T::AccountId> for Pallet<T, I> {
  fn transfer(
    asset: Self::AssetId,
    source: &T::AccountId,
    dest: &T::AccountId,
    amount: T::Balance,
    keep_alive: bool,
  ) -> Result<T::Balance, DispatchError> {
    let f = TransferFlags {
      keep_alive,
      best_effort: false,
      burn_dust: false,
    };
    Self::do_transfer(asset, source, dest, amount, None, f)
  }
}

impl<T: Config<I>, I: 'static> fungibles::Unbalanced<T::AccountId> for Pallet<T, I> {
  fn set_balance(_: Self::AssetId, _: &T::AccountId, _: Self::Balance) -> DispatchResult {
    unreachable!("set_balance is not used if other functions are impl'd");
  }
  fn set_total_issuance(id: T::AssetId, amount: Self::Balance) {
    Asset::<T, I>::mutate_exists(id, |maybe_asset| {
      if let Some(ref mut asset) = maybe_asset {
        asset.supply = amount
      }
    });
  }
  fn decrease_balance(
    asset: T::AssetId,
    who: &T::AccountId,
    amount: Self::Balance,
  ) -> Result<Self::Balance, DispatchError> {
    let f = DebitFlags {
      keep_alive: false,
      best_effort: false,
    };
    Self::decrease_balance(asset, who, amount, f, |_, _| Ok(()))
  }
  fn decrease_balance_at_most(
    asset: T::AssetId,
    who: &T::AccountId,
    amount: Self::Balance,
  ) -> Self::Balance {
    let f = DebitFlags {
      keep_alive: false,
      best_effort: true,
    };
    Self::decrease_balance(asset, who, amount, f, |_, _| Ok(())).unwrap_or(Zero::zero())
  }
  fn increase_balance(
    asset: T::AssetId,
    who: &T::AccountId,
    amount: Self::Balance,
  ) -> Result<Self::Balance, DispatchError> {
    Self::increase_balance(asset, who, amount, |_| Ok(()))?;
    Ok(amount)
  }
  fn increase_balance_at_most(
    asset: T::AssetId,
    who: &T::AccountId,
    amount: Self::Balance,
  ) -> Self::Balance {
    match Self::increase_balance(asset, who, amount, |_| Ok(())) {
      Ok(()) => amount,
      Err(_) => Zero::zero(),
    }
  }
}

impl<T: Config<I>, I: 'static> fungibles::Create<T::AccountId> for Pallet<T, I> {
  fn create(
    id: T::AssetId,
    admin: T::AccountId,
    is_sufficient: bool,
    min_balance: Self::Balance,
  ) -> DispatchResult {
    Self::do_force_create(id, admin, is_sufficient, min_balance)
  }
}

impl<T: Config<I>, I: 'static> fungibles::Destroy<T::AccountId> for Pallet<T, I> {
  type DestroyWitness = DestroyWitness;

  fn get_destroy_witness(asset: &T::AssetId) -> Option<Self::DestroyWitness> {
    Asset::<T, I>::get(asset).map(|asset_details| asset_details.destroy_witness())
  }

  fn destroy(
    id: T::AssetId,
    witness: Self::DestroyWitness,
    maybe_check_owner: Option<T::AccountId>,
  ) -> Result<Self::DestroyWitness, DispatchError> {
    Self::do_destroy(id, witness, maybe_check_owner)
  }
}

impl<T: Config<I>, I: 'static> fungibles::metadata::Inspect<<T as SystemConfig>::AccountId>
  for Pallet<T, I>
{
  fn name(asset: T::AssetId) -> Vec<u8> {
    Metadata::<T, I>::get(asset).name.to_vec()
  }

  fn symbol(asset: T::AssetId) -> Vec<u8> {
    Metadata::<T, I>::get(asset).symbol.to_vec()
  }

  fn decimals(asset: T::AssetId) -> u8 {
    Metadata::<T, I>::get(asset).decimals
  }
}

impl<T: Config<I>, I: 'static> fungibles::metadata::Mutate<<T as SystemConfig>::AccountId>
  for Pallet<T, I>
{
  fn set(
    asset: T::AssetId,
    from: &<T as SystemConfig>::AccountId,
    name: Vec<u8>,
    symbol: Vec<u8>,
    decimals: u8,
  ) -> DispatchResult {
    Self::do_set_metadata(asset, from, name, symbol, decimals)
  }
}

impl<T: Config<I>, I: 'static> fungibles::approvals::Inspect<<T as SystemConfig>::AccountId>
  for Pallet<T, I>
{
  // Check the amount approved to be spent by an owner to a delegate
  fn allowance(
    asset: T::AssetId,
    owner: &<T as SystemConfig>::AccountId,
    delegate: &<T as SystemConfig>::AccountId,
  ) -> T::Balance {
    Approvals::<T, I>::get((asset, &owner, &delegate))
      .map(|x| x.amount)
      .unwrap_or_else(Zero::zero)
  }
}

impl<T: Config<I>, I: 'static> fungibles::approvals::Mutate<<T as SystemConfig>::AccountId>
  for Pallet<T, I>
{
  fn approve(
    asset: T::AssetId,
    owner: &<T as SystemConfig>::AccountId,
    delegate: &<T as SystemConfig>::AccountId,
    amount: T::Balance,
  ) -> DispatchResult {
    Self::do_approve_transfer(asset, owner, delegate, amount)
  }

  // Aprove spending tokens from a given account
  fn transfer_from(
    asset: T::AssetId,
    owner: &<T as SystemConfig>::AccountId,
    delegate: &<T as SystemConfig>::AccountId,
    dest: &<T as SystemConfig>::AccountId,
    amount: T::Balance,
  ) -> DispatchResult {
    Self::do_transfer_approved(asset, owner, delegate, dest, amount)
  }
}

impl<T: Config<I>, I: 'static> fungibles::InspectHold<T::AccountId> for Pallet<T, I> {
  fn balance_on_hold(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
    Account::<T, I>::get(who, asset)
      .map(|a| a.reserved)
      .unwrap_or_default()
  }

  fn can_hold(asset: Self::AssetId, who: &T::AccountId, amount: Self::Balance) -> bool {
    Pallet::<T, I>::can_decrease(asset, who, amount, true)
      .into_result()
      .is_ok()
  }
}

impl<T: Config<I>, I: 'static> fungibles::MutateHold<T::AccountId> for Pallet<T, I> {
  fn hold(asset: Self::AssetId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
    Self::do_hold(asset, who, amount)?;
    Ok(())
  }

  fn release(
    asset: Self::AssetId,
    who: &T::AccountId,
    amount: Self::Balance,
    _best_effort: bool,
  ) -> Result<Self::Balance, DispatchError> {
    Self::do_release(asset, who, amount)
  }

  // Transfer held funds to an account
  fn transfer_held(
    asset: Self::AssetId,
    source: &T::AccountId,
    dest: &T::AccountId,
    amount: Self::Balance,
    best_effort: bool,
    on_hold: bool,
  ) -> Result<Self::Balance, DispatchError> {
    let f = TransferFlags {
      keep_alive: true,
      best_effort,
      burn_dust: false,
    };
    Self::do_transfer_held(asset, source, dest, amount, false, on_hold, f)
  }
}
