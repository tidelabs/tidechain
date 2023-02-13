use crate as pallet_tidefi_stake;
use frame_support::{
  pallet_prelude::ValueQuery,
  storage_alias,
  traits::{Get, GetStorageVersion, PalletInfoAccess},
  weights::Weight,
};
use hex_literal::hex;
use sp_std::vec::Vec;
use tidefi_primitives::Hash;

// old storage queue
#[storage_alias]
type UnstakeQueue<T: pallet_tidefi_stake::Config> = StorageValue<
  crate::Pallet<T>,
  Vec<(
    <T as frame_system::Config>::AccountId,
    Hash,
    <T as frame_system::Config>::BlockNumber,
  )>,
  ValueQuery,
>;

pub fn migrate<T: pallet_tidefi_stake::Config, P: GetStorageVersion + PalletInfoAccess>() -> Weight
where
  <T as frame_system::Config>::AccountId: From<[u8; 32]>,
{
  let mut weight = T::DbWeight::get().reads_writes(3, 2);

  let on_chain_storage_version = <P as GetStorageVersion>::on_chain_storage_version();
  log::info!(
    target: "runtime::tidefi-stake",
    "Running migration to v2 for tidefi-stake with storage version {:?}",
    on_chain_storage_version,
  );
  if on_chain_storage_version < 2 {
    // update `PendingStoredSessions`
    let stored_session_size = pallet_tidefi_stake::PendingStoredSessions::<T>::count();
    let mut unstake_queue_size: u32 = 0;

    pallet_tidefi_stake::PendingStoredSessions::<T>::translate(|_, _: ()| Some(Default::default()));

    // migrate unstake queue
    UnstakeQueue::<T>::take()
      .iter()
      .for_each(|(account_id, hash, expected_end)| {
        pallet_tidefi_stake::QueueUnstake::<T>::insert(hash, (account_id, expected_end));
        unstake_queue_size += 1;
      });

    // add default operator
    pallet_tidefi_stake::OperatorAccountId::<T>::put::<T::AccountId>(
      hex!["8e14d1ac896ea00e18d855588ee13103449cc6e41d4b0173d917cabea84bdb08"].into(),
    );

    weight = weight
      .saturating_add(
        T::DbWeight::get().reads_writes(stored_session_size.into(), stored_session_size.into()),
      )
      .saturating_add(
        T::DbWeight::get().reads_writes(unstake_queue_size.into(), unstake_queue_size.into()),
      );

    log::info!(
      target: "runtime::tidefi-stake",
      "Migrated {} sessions and {} unstake queue items successfully.",
      stored_session_size,
      unstake_queue_size
    );
  } else {
    log::warn!(
      target: "runtime::tidefi-stake",
      "Attempted to apply migration to v2 but failed because storage version is {:?}",
      on_chain_storage_version,
    );
  }

  weight
}
