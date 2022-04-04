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

use super::*;
use frame_support::{
  log,
  traits::{Get, GetStorageVersion, PalletInfoAccess, StorageVersion},
};
use tidefi_primitives::{pallet::SecurityExt, ActiveEraInfo};

/// Migrate the pallet storage to v1.
pub fn migrate_to_v1<T: Config, P: GetStorageVersion + PalletInfoAccess>(
) -> frame_support::weights::Weight {
  let on_chain_storage_version = <P as GetStorageVersion>::on_chain_storage_version();
  log!(
    info,
    "Running migration storage v1 with storage version {:?}",
    on_chain_storage_version,
  );

  if on_chain_storage_version < 1 {
    // set default staking periods
    CurrentSession::<T>::put(1);
    // reset everything, we don't really bother with previous stake / rewards as everything changed
    EraTotalFees::<T>::remove_all(None);
    SessionTotalFees::<T>::remove_all(None);
    AccountFees::<T>::remove_all(None);
    // reset era
    ActiveEra::<T>::put(ActiveEraInfo::<T::BlockNumber> {
      index: 1,
      // Set new active era start in next `on_finalize`. To guarantee usage of `Time`
      start_block: None,
      start_session_index: None,
      last_session_block: None,
      start: None,
    });

    // update on-chain storage version
    StorageVersion::new(1).put::<P>();
    log!(
      info,
      "Running migration storage v1 with storage version {:?} was complete",
      on_chain_storage_version,
    );
    // return migration weights
    T::DbWeight::get().reads_writes(1, 6)
  } else {
    log!(
      info,
      "Attempted to apply migration to v1 but failed because storage version is {:?}",
      on_chain_storage_version,
    );
    T::DbWeight::get().reads(1)
  }
}
