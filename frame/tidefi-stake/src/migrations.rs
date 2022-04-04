use super::*;
use frame_support::{
  log,
  traits::{Get, GetStorageVersion, PalletInfoAccess, StorageVersion},
  BoundedVec,
};
use sp_runtime::Percent;
use sp_std::vec;
use tidefi_primitives::pallet::SecurityExt;

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
    StakingPool::<T>::remove_all(None);
    AccountStakes::<T>::remove_all();

    // set default staking periods
    let bounded_periods: BoundedVec<(T::BlockNumber, Percent), T::StakingRewardCap> = vec![
      (T::BlockNumber::from(150_u32), Percent::from_parts(1)),
      (
        T::BlockNumber::from(14400_u32 * 15_u32),
        Percent::from_parts(2),
      ),
      (
        T::BlockNumber::from(14400_u32 * 30_u32),
        Percent::from_parts(3),
      ),
      (
        T::BlockNumber::from(14400_u32 * 60_u32),
        Percent::from_parts(4),
      ),
      (
        T::BlockNumber::from(14400_u32 * 90_u32),
        Percent::from_parts(5),
      ),
    ]
    .try_into()
    .expect("too much periods");

    StakingPeriodRewards::<T>::put(bounded_periods);

    // set defaut staking fee (1%)
    UnstakeFee::<T>::put(Percent::from_parts(1));

    // update on-chain storage version
    StorageVersion::new(1).put::<P>();
    log!(
      info,
      "Running migration storage v1 with storage version {:?} was complete",
      on_chain_storage_version,
    );
    // return migration weights
    T::DbWeight::get().reads_writes(1, 5)
  } else {
    log!(
      info,
      "Attempted to apply migration to v1 but failed because storage version is {:?}",
      on_chain_storage_version,
    );
    T::DbWeight::get().reads(1)
  }
}
