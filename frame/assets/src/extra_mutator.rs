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

//! Datatype for easy mutation of the extra "sidecar" data.

use super::*;

/// A `mutator` type allowing inspection and possible modification of the extra "sidecar" data.
///
/// This may be used as a `Deref` for the pallet's extra data. If mutated (using `DerefMut`), then
/// any uncommitted changes (see `commit` function) will be automatically committed to storage when
/// dropped. Changes, even after committed, may be reverted to their original values with the
/// `revert` function.
pub struct ExtraMutator<T: Config<I>, I: 'static = ()> {
  id: T::AssetId,
  who: T::AccountId,
  original: T::Extra,
  pending: Option<T::Extra>,
}

impl<T: Config<I>, I: 'static> Drop for ExtraMutator<T, I> {
  fn drop(&mut self) {
    debug_assert!(
      self.commit().is_ok(),
      "attempt to write to non-existent asset account"
    );
  }
}

impl<T: Config<I>, I: 'static> sp_std::ops::Deref for ExtraMutator<T, I> {
  type Target = T::Extra;
  fn deref(&self) -> &T::Extra {
    match self.pending {
      Some(ref value) => value,
      None => &self.original,
    }
  }
}

impl<T: Config<I>, I: 'static> sp_std::ops::DerefMut for ExtraMutator<T, I> {
  fn deref_mut(&mut self) -> &mut T::Extra {
    if self.pending.is_none() {
      self.pending = Some(self.original.clone());
    }
    self.pending.as_mut().unwrap()
  }
}

impl<T: Config<I>, I: 'static> ExtraMutator<T, I> {
  pub(super) fn maybe_new(
    id: T::AssetId,
    who: impl sp_std::borrow::Borrow<T::AccountId>,
  ) -> Option<ExtraMutator<T, I>> {
    if let Some(a) = Account::<T, I>::get(who.borrow(), id) {
      Some(ExtraMutator::<T, I> {
        id,
        who: who.borrow().clone(),
        original: a.extra,
        pending: None,
      })
    } else {
      None
    }
  }

  /// Commit any changes to storage.
  pub fn commit(&mut self) -> Result<(), ()> {
    if let Some(extra) = self.pending.take() {
      Account::<T, I>::try_mutate(self.who.borrow(), self.id, |maybe_account| {
        maybe_account
          .as_mut()
          .ok_or(())
          .map(|account| account.extra = extra)
      })
    } else {
      Ok(())
    }
  }

  /// Revert any changes, even those already committed by `self` and drop self.
  pub fn revert(mut self) -> Result<(), ()> {
    self.pending = None;
    Account::<T, I>::try_mutate(self.who.borrow(), self.id, |maybe_account| {
      maybe_account
        .as_mut()
        .ok_or(())
        .map(|account| account.extra = self.original.clone())
    })
  }
}
