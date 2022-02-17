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
    "Running migration storage v1 for uniques with storage version {:?}",
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
