#![cfg_attr(not(feature = "std"), no_std)]

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
  use frame_support::{inherent::Vec, pallet_prelude::*, PalletId};
  use frame_system::{pallet_prelude::*, RawOrigin};
  use sp_core::{H256, U256};
  use sp_runtime::traits::{AccountIdConversion, Saturating, StaticLookup};
  use sp_std::convert::TryInto;
  use tidefi_primitives::{
    pallet::AssetRegistryExt, AccountId, AssetId, Balance, BlockNumber, CurrencyId,
    CurrencyMetadata, Hash, StatusCode,
  };

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config:
    frame_system::Config + pallet_assets::Config<AssetId = AssetId, Balance = Balance>
  {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    #[pallet::constant]
    type AssetRegistryPalletId: Get<PalletId>;
    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  #[pallet::storage]
  #[pallet::getter(fn assets)]
  /// Asset mapping [currency_id, is_enabled]
  pub type Assets<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, bool, OptionQuery>;

  /// Assets Account ID owner
  #[pallet::storage]
  #[pallet::getter(fn account_id)]
  pub type AssetRegistryAccountId<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    /// [currency_id, name, symbol, decimals]
    pub assets: Vec<(CurrencyId, Vec<u8>, Vec<u8>, u8)>,
    pub account: T::AccountId,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        assets: Vec::new(),
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
      for (currency_id, name, symbol, decimals) in self.assets.clone() {
        // If it's a wrapped token, register it with pallet_assets
        if let CurrencyId::Wrapped(asset_id) = currency_id {
          let _ = Pallet::<T>::register_asset(asset_id, name, symbol, decimals, 1);
        }
        // Insert inside our local map
        Assets::<T>::insert(currency_id, true);
      }
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Asset was registered. [currency_id]
    Registered(CurrencyId),
    /// Asset was updated. [currency_id]
    Updated(CurrencyId),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// The access to the Asset registry pallet is not allowed for this account ID.
    AccessDenied,
    /// Invalid asset name or symbol.
    AssetNotFound,
    /// Asset ID is not registered in the asset-registry.
    AssetNotRegistered,
    /// Asset ID status is already the same as requested.
    NoStatusChangeRequested,
    /// Asset is already registered.
    AssetAlreadyRegistered,
    /// Invalid CurrencyId
    CurrencyIdNotValid,
  }

  // Dispatchable functions allows users to interact with the pallet and invoke state changes.
  // These functions materialize as "extrinsics", which are often compared to transactions.
  // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Register a new asset.
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
        ensure_signed(origin)? == Self::account_id(),
        Error::<T>::AccessDenied
      );

      // 2. Make sure the asset isn't already registered
      ensure!(
        Self::assets(currency_id).is_none(),
        Error::<T>::AssetAlreadyRegistered
      );

      // 3. If it's a wrapped token, let's register it with pallet_assets
      if let CurrencyId::Wrapped(asset_id) = currency_id {
        Self::register_asset(asset_id, name, symbol, decimals, existential_deposit)?;
      }

      // 4. Register local store
      Assets::<T>::insert(currency_id, true);

      // 5. Emit new registered currency
      Self::deposit_event(<Event<T>>::Registered(currency_id));

      Ok(().into())
    }

    /// Update asset status.
    #[pallet::weight(<T as Config>::WeightInfo::set_status())]
    pub fn set_status(
      origin: OriginFor<T>,
      currency_id: CurrencyId,
      is_enabled: bool,
    ) -> DispatchResult {
      // 1. Make sure it's signed from the asset-registry owner
      ensure!(
        ensure_signed(origin)? == Self::account_id(),
        Error::<T>::AccessDenied
      );

      // 2. Make sure the currency is already registered
      ensure!(
        Self::assets(currency_id).is_some(),
        Error::<T>::AssetNotRegistered
      );

      // 3. Make sure the status will change
      ensure!(
        Self::assets(currency_id) == Some(!is_enabled),
        Error::<T>::NoStatusChangeRequested
      );

      // 4. If it's wrapped asset, freeze/unfreeze at the chain level
      if let CurrencyId::Wrapped(asset_id) = currency_id {
        match is_enabled {
          true => {
            // unfreeze asset
            pallet_assets::Pallet::<T>::thaw_asset(
              RawOrigin::Signed(Self::account_id()).into(),
              asset_id,
            )?;
          }
          false => {
            // freeze asset
            pallet_assets::Pallet::<T>::freeze_asset(
              RawOrigin::Signed(Self::account_id()).into(),
              asset_id,
            )?;
          }
        };
      }

      // 5. Mutate local storage for quick reference
      <Assets<T>>::mutate(currency_id, |asset| {
        *asset = Some(is_enabled);
      });

      Ok(().into())
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
        T::Lookup::unlookup(Self::account_id()),
        true,
        existential_deposit,
      )?;

      // 2. Set metadata
      pallet_assets::Pallet::<T>::force_set_metadata(
        RawOrigin::Signed(Self::account_id()).into(),
        asset_id,
        name,
        symbol,
        decimals,
        false,
      )?;

      Ok(())
    }
  }

  impl<T: Config> AssetRegistryExt for Pallet<T> {
    fn is_enabled(currency_id: CurrencyId) -> bool {
      Self::assets(currency_id).unwrap_or(false)
    }
  }
}
