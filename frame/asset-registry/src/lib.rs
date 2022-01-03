#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
  use super::*;
  use frame_support::{
    inherent::Vec,
    pallet_prelude::*,
    traits::fungibles::{Inspect, Mutate, Transfer},
    PalletId,
  };
  use frame_system::{pallet_prelude::*, RawOrigin};
  use sp_runtime::traits::{AccountIdConversion, StaticLookup};
  use sp_std::vec;
  use tidefi_primitives::{
    pallet::AssetRegistryExt, AssetId, Balance, BalanceInfo, CurrencyId, CurrencyMetadata,
  };

  /// Asset registry configuration
  #[pallet::config]
  pub trait Config:
    frame_system::Config + pallet_assets::Config<AssetId = AssetId, Balance = Balance>
  {
    /// Events
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    /// Pallet ID
    #[pallet::constant]
    type AssetRegistryPalletId: Get<PalletId>;

    /// Weights
    type WeightInfo: WeightInfo;

    /// Currency wrapr
    type CurrencyWrapr: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  /// Assets Account ID owner
  #[pallet::storage]
  #[pallet::getter(fn account_id)]
  pub type AssetRegistryAccountId<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

  /// Genesis configuration
  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    /// Assets to create on initialization
    /// \[currency_id, name, symbol, decimals\]
    pub assets: Vec<(
      CurrencyId,
      Vec<u8>,
      Vec<u8>,
      u8,
      Vec<(T::AccountId, T::Balance)>,
    )>,
    /// Assets owner
    /// Only this account can modify storage on this pallet.
    pub account: T::AccountId,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        // empty assets by default
        assets: Vec::new(),
        // We use pallet account ID by default,
        // but should always be set in the genesis config.
        account: T::AssetRegistryPalletId::get().into_account(),
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      // 1. Save asset registry account id
      AssetRegistryAccountId::<T>::put(self.account.clone());

      // 2. Loop trough all currency defined in our genesis config
      for (currency_id, name, symbol, decimals, pre_filled_account) in self.assets.clone() {
        // If it's a wrapped token, register it with pallet_assets
        if let CurrencyId::Wrapped(asset_id) = currency_id {
          let _ = Pallet::<T>::register_asset(asset_id, name, symbol, decimals, 1);
        }

        for (account_id, mint_amount) in pre_filled_account {
          let _ = T::CurrencyWrapr::mint_into(currency_id, &account_id, mint_amount);
        }
      }
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Asset was registered. \[currency_id\]
    Registered(CurrencyId),
    /// Asset was updated. \[currency_id, is_enabled\]
    StatusChanged(CurrencyId, bool),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// The access to the Asset registry pallet is not allowed for this account ID.
    AccessDenied,
    /// Asset ID is not registered in the asset-registry.
    AssetNotRegistered,
    /// Asset ID status is already the same as requested.
    NoStatusChangeRequested,
    /// Asset is already registered.
    AssetAlreadyRegistered,
    /// Invalid CurrencyId
    CurrencyIdNotValid,
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Register new asset on chain.
    ///
    /// - `currency_id`: The currency ID to register
    /// - `name`: Currency name. Ex: `Bitcoin`
    /// - `symbol`: Currency symbol. Ex: `BTC`
    /// - `decimals`: Number of decimals for the asset. Ex: `8`
    /// - `existential_deposit`: Number of token required to keep the balance alive. Ex: `1`
    ///
    /// Emits `Registered` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as Config>::WeightInfo::set_status())]
    pub fn register(
      origin: OriginFor<T>,
      currency_id: CurrencyId,
      name: Vec<u8>,
      symbol: Vec<u8>,
      decimals: u8,
      existential_deposit: <T as pallet_assets::Config>::Balance,
    ) -> DispatchResult {
      // 1. Make sure it's signed from the asset-registry owner
      ensure!(
        Some(ensure_signed(origin)?) == Self::account_id(),
        Error::<T>::AccessDenied
      );

      // 2. Make sure the asset isn't already registered
      ensure!(
        !Self::is_currency_exist(currency_id),
        Error::<T>::AssetAlreadyRegistered
      );

      // 3. If it's a wrapped token, let's register it with pallet_assets
      if let CurrencyId::Wrapped(asset_id) = currency_id {
        Self::register_asset(asset_id, name, symbol, decimals, existential_deposit)?;
      }

      // 5. Emit new registered currency
      Self::deposit_event(<Event<T>>::Registered(currency_id));

      Ok(())
    }

    /// Update asset status.
    ///
    /// - `currency_id`: The currency ID to register
    /// - `is_enabled`: Is the currency enabled on chain?
    ///
    /// Emits `StatusChanged` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as Config>::WeightInfo::set_status())]
    pub fn set_status(
      origin: OriginFor<T>,
      currency_id: CurrencyId,
      is_enabled: bool,
    ) -> DispatchResult {
      // 1. Make sure it's signed from the asset-registry owner
      ensure!(
        Some(ensure_signed(origin)?) == Self::account_id(),
        Error::<T>::AccessDenied
      );

      // 2. Make sure the currency is already registered
      ensure!(
        Self::is_currency_exist(currency_id),
        Error::<T>::AssetNotRegistered
      );

      // 3. Freeze/unfreeze at the chain level, do nothing if
      // we requested a TIDE freeze
      if let CurrencyId::Wrapped(asset_id) = currency_id {
        match is_enabled {
          true => {
            // unfreeze asset
            pallet_assets::Pallet::<T>::thaw_asset(
              RawOrigin::Signed(T::AssetRegistryPalletId::get().into_account()).into(),
              asset_id,
            )?;
          }
          false => {
            // freeze asset
            pallet_assets::Pallet::<T>::freeze_asset(
              RawOrigin::Signed(T::AssetRegistryPalletId::get().into_account()).into(),
              asset_id,
            )?;
          }
        };
      }

      // 4. Emit new registered currency
      Self::deposit_event(<Event<T>>::StatusChanged(currency_id, is_enabled));

      Ok(())
    }
  }

  impl<T: Config> Pallet<T> {
    fn register_asset(
      asset_id: T::AssetId,
      name: Vec<u8>,
      symbol: Vec<u8>,
      decimals: u8,
      existential_deposit: <T as pallet_assets::Config>::Balance,
    ) -> Result<(), DispatchError> {
      // 1. Create asset
      pallet_assets::Pallet::<T>::force_create(
        RawOrigin::Root.into(),
        asset_id,
        // make the pallet account id the owner, so only this pallet can handle the funds.
        T::Lookup::unlookup(T::AssetRegistryPalletId::get().into_account()),
        true,
        existential_deposit,
      )?;

      // 2. Set metadata
      pallet_assets::Pallet::<T>::force_set_metadata(
        RawOrigin::Signed(T::AssetRegistryPalletId::get().into_account()).into(),
        asset_id,
        name,
        symbol,
        decimals,
        false,
      )?;

      Ok(())
    }

    pub fn is_currency_exist(currency_id: CurrencyId) -> bool {
      match currency_id {
        // tide always exist
        CurrencyId::Tide => true,
        CurrencyId::Wrapped(asset_id) => {
          pallet_assets::Pallet::<T>::asset_details(asset_id).is_some()
        }
      }
    }

    pub fn get_account_balance(
      account_id: &T::AccountId,
      asset_id: CurrencyId,
    ) -> Result<BalanceInfo, DispatchError> {
      let balance = T::CurrencyWrapr::balance(asset_id, account_id);
      Ok(BalanceInfo { amount: balance })
    }

    pub fn get_assets() -> Result<Vec<(CurrencyId, CurrencyMetadata)>, DispatchError> {
      let mut final_assets = vec![(
        CurrencyId::Tide,
        CurrencyMetadata {
          name: "Tide".into(),
          symbol: "TIDE".into(),
          decimals: 12,
          is_frozen: false,
        },
      )];

      let mut asset_metadatas = pallet_assets::Metadata::<T>::iter()
        .map(|(asset_id, asset_metadata)| {
          (
            CurrencyId::Wrapped(asset_id),
            CurrencyMetadata {
              name: asset_metadata.name.into(),
              symbol: asset_metadata.symbol.into(),
              decimals: asset_metadata.decimals,
              is_frozen: asset_metadata.is_frozen,
            },
          )
        })
        .collect();

      final_assets.append(&mut asset_metadatas);

      Ok(final_assets)
    }

    pub fn get_account_balances(
      account_id: &T::AccountId,
    ) -> Result<Vec<(CurrencyId, BalanceInfo)>, DispatchError> {
      let mut final_balances = vec![(
        CurrencyId::Tide,
        Self::get_account_balance(account_id, CurrencyId::Tide)?,
      )];
      let mut asset_balances = pallet_assets::Account::<T>::iter_prefix(account_id)
        .map(|(asset_id, balance)| {
          (
            CurrencyId::Wrapped(asset_id),
            BalanceInfo {
              amount: balance.balance,
            },
          )
        })
        .collect();
      final_balances.append(&mut asset_balances);
      Ok(final_balances)
    }
  }

  impl<T: Config> AssetRegistryExt for Pallet<T> {
    fn is_currency_enabled(currency_id: CurrencyId) -> bool {
      match currency_id {
        // we can't disable tide
        CurrencyId::Tide => true,
        CurrencyId::Wrapped(asset_id) => pallet_assets::Pallet::<T>::asset_details(asset_id)
          .map(|detail| !detail.is_frozen)
          .unwrap_or(false),
      }
    }
  }
}
