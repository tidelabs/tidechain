use crate as pallet_tidefi_stake;
use frame_support::{
  pallet_prelude::ValueQuery,
  storage_alias,
  traits::{Get, GetStorageVersion, PalletInfoAccess, StorageVersion},
  weights::Weight,
};
use hex_literal::hex;
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
use tidefi_primitives::{assets::Asset, Balance, CurrencyId, Hash};

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
    let mut staking_pool_size: BTreeMap<CurrencyId, Balance> = BTreeMap::new();
    let stored_session_size = pallet_tidefi_stake::PendingStoredSessions::<T>::count();
    let mut unstake_queue_size: u32 = 0;

    // update `PendingStoredSessions`
    pallet_tidefi_stake::PendingStoredSessions::<T>::translate(|_, _: ()| Some(Default::default()));

    // migrate unstake queue
    UnstakeQueue::<T>::take()
      .iter()
      .for_each(|(account_id, hash, expected_end)| {
        pallet_tidefi_stake::QueueUnstake::<T>::insert(hash, (account_id, expected_end));
        unstake_queue_size += 1;
      });
    UnstakeQueue::<T>::kill();

    // add default operator
    pallet_tidefi_stake::OperatorAccountId::<T>::put::<T::AccountId>(
      hex!["8e14d1ac896ea00e18d855588ee13103449cc6e41d4b0173d917cabea84bdb08"].into(),
    );

    let expected_stored_session_rw = stored_session_size.saturating_mul(2).into();
    let expected_unstake_queue_rw = unstake_queue_size.saturating_mul(2).into();
    weight = weight
      .saturating_add(
        T::DbWeight::get().reads_writes(expected_stored_session_rw, expected_stored_session_rw),
      )
      .saturating_add(
        T::DbWeight::get().reads_writes(expected_unstake_queue_rw, expected_unstake_queue_rw),
      );

    // recompute staking pool including principal inside a btreemap
    pallet_tidefi_stake::AccountStakes::<T>::iter().for_each(|(_, stakes)| {
      stakes.iter().for_each(|stake| {
        if let Some(principal) = staking_pool_size.get_mut(&stake.currency_id) {
          *principal = principal.saturating_add(stake.principal);
        } else {
          staking_pool_size.insert(stake.currency_id, stake.principal);
        }
        weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
      });
    });

    // update our staking pool storage once everything has been computed
    pallet_tidefi_stake::StakingPool::<T>::translate(|currency_id, balance: Balance| {
      let asset: Asset = currency_id.try_into().expect("valid currency");
      let new_balance = staking_pool_size.get(&currency_id).unwrap_or(&balance);

      log::info!(
        target: "runtime::tidefi-stake",
        "Update {} staking pool from {} to {} (including rewards)",
        asset.symbol(),
        balance,
        new_balance
      );

      weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
      Some(*new_balance)
    });

    StorageVersion::new(2).put::<P>();

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

pub fn post_migration<T: pallet_tidefi_stake::Config, P: GetStorageVersion + PalletInfoAccess>() {
  assert_eq!(<P as GetStorageVersion>::on_chain_storage_version(), 2);
}
