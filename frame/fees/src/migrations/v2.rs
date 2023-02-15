use crate as pallet_fees;
use frame_support::{
  traits::{
    fungibles::{Inspect, Transfer},
    Get, GetStorageVersion, PalletInfoAccess, StorageVersion,
  },
  weights::Weight,
};
use hex_literal::hex;
use sp_runtime::traits::AccountIdConversion;
use sp_std::collections::btree_map::BTreeMap;
use tidefi_primitives::{assets::Asset, Balance, CurrencyId};

pub fn migrate<
  T: pallet_fees::Config + pallet_tidefi_stake::Config + pallet_assets::Config,
  P: GetStorageVersion + PalletInfoAccess,
>() -> Weight
where
  <T as frame_system::Config>::AccountId: From<[u8; 32]>,
{
  let on_chain_storage_version = <P as GetStorageVersion>::on_chain_storage_version();
  log::info!(
    target: "runtime::fees",
    "Running migration to v2 for fees with storage version {:?}",
    on_chain_storage_version,
  );
  if on_chain_storage_version < 2 {
    let mut staking_pool_size: BTreeMap<CurrencyId, (Balance, Balance)> = BTreeMap::new();

    let staking_pool_account_id: T::AccountId =
      <T as pallet_tidefi_stake::Config>::StakePalletId::get().into_account_truncating();
    // we default our operator, if staking migration isnt completed for any reason, we could
    // have the correct account id for our migration
    let operator_account_id: T::AccountId = pallet_tidefi_stake::OperatorAccountId::<T>::get()
      .unwrap_or(hex!["8e14d1ac896ea00e18d855588ee13103449cc6e41d4b0173d917cabea84bdb08"].into());
    let fees_account_id: T::AccountId =
      <T as pallet_fees::Config>::FeesPalletId::get().into_account_truncating();

    pallet_tidefi_stake::AccountStakes::<T>::iter().for_each(|(_, stakes)| {
      stakes.iter().for_each(|stake| {
        if let Some((principal, initial)) = staking_pool_size.get_mut(&stake.currency_id) {
          *principal = principal.saturating_add(stake.principal);
          *initial = initial.saturating_add(stake.initial_balance);
        } else {
          staking_pool_size.insert(stake.currency_id, (stake.principal, stake.initial_balance));
        }
      });
    });

    log::info!(
      target: "runtime::fees",
      "Expected staking pool {:?}",
      staking_pool_size.iter().map(|(currency_id, b)| {
        let asset: Asset = currency_id.clone().try_into().expect("valid currency");
        let asset_symbol = asset.symbol();
        (asset_symbol, b)
      }),
    );

    // transfer assets to staking pool or to operator account
    pallet_tidefi_stake::StakingCurrencyMeta::<T>::iter_keys().for_each(|currency_id| {
      let asset: Asset = currency_id.try_into().expect("valid currency");
      let asset_symbol = asset.symbol();

      // balance of the staking pool
      let staking_balance = <T as pallet_fees::Config>::CurrencyTidefi::reducible_balance(
        currency_id,
        &staking_pool_account_id,
        false,
      );

      // balance of the fees pallet
      let fees_balance = <T as pallet_fees::Config>::CurrencyTidefi::reducible_balance(
        currency_id,
        &fees_account_id,
        false,
      );

      if let Some((principal, _)) = staking_pool_size.get(&currency_id) {
        if staking_balance < *principal {
          let missing_funds = principal.saturating_sub(staking_balance);
          if missing_funds > 0 && missing_funds <= fees_balance {
            let result = <T as pallet_fees::Config>::CurrencyTidefi::transfer(
              currency_id,
              &fees_account_id,
              &staking_pool_account_id,
              missing_funds,
              false,
            );

            log::info!(
              target: "runtime::fees",
              "Transfered {} {} from fees to staking pallet; outcome: {:?}",
              missing_funds,
              asset_symbol,
              result
            );
          } else {
            log::info!(
              target: "runtime::fees",
              "Staking pool for {} balance match {}={}",
              asset_symbol,
              staking_balance,
              principal
            );
          }

          // if funds left, send to operator
          let funds_left = fees_balance.saturating_sub(missing_funds);
          if funds_left > 0 {
            let result = <T as pallet_fees::Config>::CurrencyTidefi::transfer(
              currency_id,
              &fees_account_id,
              &operator_account_id,
              funds_left,
              false,
            );

            log::info!(
              target: "runtime::fees",
              "Transfered {} {} from fees to operator account; outcome: {:?}",
              funds_left,
              asset_symbol,
              result
            );
          } else {
            log::info!(
              target: "runtime::fees",
              "No funds left for {} in the fees pallet for the operator",
              asset_symbol
            );
          }
        } else {
          log::info!(
            target: "runtime::fees",
            "Staking pool for {} balance match {}={}",
            asset_symbol,
            staking_balance,
            principal
          );
        }
      };
    });

    // transfer TDFY funds of the fee pallet to the operator
    let tdfy_fees_pallet_balance = <T as pallet_fees::Config>::CurrencyTidefi::reducible_balance(
      CurrencyId::Tdfy,
      &fees_account_id,
      true,
    );
    if tdfy_fees_pallet_balance > 0 {
      let result = <T as pallet_fees::Config>::CurrencyTidefi::transfer(
        CurrencyId::Tdfy,
        &fees_account_id,
        &operator_account_id,
        tdfy_fees_pallet_balance,
        true,
      );

      log::info!(
        target: "runtime::fees",
        "Transfered {} TDFY from fees to operator account; outcome: {:?}",
        tdfy_fees_pallet_balance,
        result
      );
    }

    StorageVersion::new(2).put::<P>();

    log::info!(
      target: "runtime::fees",
      "Migrated fees balance successfully."
    );

    <T as frame_system::Config>::BlockWeights::get().max_block
  } else {
    log::warn!(
      target: "runtime::fees",
      "Attempted to apply migration to v2 but failed because storage version is {:?}",
      on_chain_storage_version,
    );
    0
  }
}

pub fn post_migration<
  T: pallet_fees::Config + pallet_tidefi_stake::Config,
  P: GetStorageVersion + PalletInfoAccess,
>() {
  assert_eq!(<P as GetStorageVersion>::on_chain_storage_version(), 2);
}
