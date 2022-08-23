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

use crate::{Call, Config};
use codec::{Decode, Encode};
use frame_support::traits::{Get, IsSubType};
use scale_info::TypeInfo;
use sp_runtime::{
  traits::{Bounded, DispatchInfoOf, SignedExtension},
  transaction_validity::{
    InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
  },
};
use sp_std::{marker::PhantomData, prelude::*};

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CheckCallLength<T: Config + Send + Sync>(PhantomData<T>);

impl<T: Config + Send + Sync> sp_std::fmt::Debug for CheckCallLength<T> {
  fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
    write!(f, "CheckCallLength")
  }
}

impl<T: Config + Send + Sync> CheckCallLength<T> {
  /// Create new `SignedExtension` to check runtime version.
  pub fn new() -> Self {
    Self(sp_std::marker::PhantomData)
  }
}

impl<T: Config + Send + Sync> SignedExtension for CheckCallLength<T>
where
  <T as Config>::Call: IsSubType<Call<T>>,
{
  const IDENTIFIER: &'static str = "CheckCallLength";
  type AccountId = T::AccountId;
  type Call = <T as Config>::Call;
  type AdditionalSigned = ();
  type Pre = ();

  fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
    Ok(())
  }

  fn pre_dispatch(
    self,
    who: &Self::AccountId,
    call: &Self::Call,
    info: &DispatchInfoOf<Self::Call>,
    len: usize,
  ) -> Result<Self::Pre, TransactionValidityError> {
    self.validate(who, call, info, len).map(|_| ())
  }

  fn validate(
    &self,
    _who: &Self::AccountId,
    call: &Self::Call,
    _info: &DispatchInfoOf<Self::Call>,
    len: usize,
  ) -> TransactionValidity {
    // check for `withdrawal`
    match call.is_sub_type() {
      Some(Call::withdrawal { .. }) => {
        let maximum_call_length = T::MaximumCallLength::get() as usize;

        // if the withdrawal transaction is too big, just drop it.
        if len > maximum_call_length {
          return InvalidTransaction::ExhaustsResources.into();
        }

        let valid_tx = ValidTransaction {
          priority: Bounded::max_value(),
          ..Default::default()
        };
        Ok(valid_tx)
      }
      _ => Ok(Default::default()),
    }
  }
}
