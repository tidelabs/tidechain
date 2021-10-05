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
  use frame_support::{inherent::Vec, pallet_prelude::*};
  use frame_system::{pallet_prelude::*, RawOrigin};
  use pallet_assets::AssetMetadata;
  use sp_core::{H256, U256};
  use sp_runtime::traits::{Saturating, StaticLookup};
  use sp_std::convert::TryInto;
  use tidefi_primitives::{
    pallet::SecurityExt, AccountId, AssetId, Balance, BlockNumber, CurrencyId, Hash, StatusCode,
  };

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config:
    frame_system::Config + pallet_assets::Config<AssetId = AssetId, Balance = Balance>
  {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    /// The origin which can work with asset-registry.
    type RegistryOrigin: EnsureOrigin<Self::Origin>;
    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  #[pallet::storage]
  #[pallet::getter(fn assets)]
  /// Details of an asset.
  pub type Assets<T: Config> = StorageMap<_, Twox64Concat, T::AssetId, bool, OptionQuery>;

  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    pub assets: Vec<AssetMetadata<T::Balance>>,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self { assets: Vec::new() }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      //OracleStatus::<T>::put(self.enabled);
      //OracleAccountId::<T>::put(self.account.clone());
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Asset was registered. \[asset_id, name, type\]
    Registered(T::AssetId),

    /// Asset was updated. \[asset_id, name, type\]
    Updated(T::AssetId),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Invalid asset name or symbol.
    AssetNotFound,
    /// Invalid asset name or symbol.
    TooLong,
    /// Asset ID is not registered in the asset-registry.
    AssetNotRegistered,
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
    ///
    /// Asset is identified by `name` and the name must not be used to
    /// register another asset.
    ///
    /// New asset is given `NextAssetId` - sequential asset id
    ///
    /// Adds mapping between `name` and assigned `asset_id` so asset id can
    /// be retrieved by name too (Note: this approach is used in AMM
    /// implementation (xyk))
    ///
    /// Emits 'Registered` event when successful.
    #[pallet::weight(<T as Config>::WeightInfo::set_status())]
    pub fn register(
      origin: OriginFor<T>,
      currency_id: CurrencyId,
      name: Vec<u8>,
      symbol: Vec<u8>,
      decimals: u8,
      existential_deposit: T::Balance,
    ) -> DispatchResult {
      T::RegistryOrigin::ensure_origin(origin)?;

      if let CurrencyId::Wrapped(asset_id) = currency_id {
        ensure!(
          Self::assets(asset_id).is_none(),
          Error::<T>::AssetAlreadyRegistered
        );

        Self::register_asset(asset_id, name, symbol, decimals, existential_deposit)?;

        return Ok(().into());
      }

      Err(Error::<T>::CurrencyIdNotValid.into())
    }
  }

  impl<T: Config> Pallet<T> {
    fn register_asset(
      asset_id: T::AssetId,
      name: Vec<u8>,
      symbol: Vec<u8>,
      decimals: u8,
      existential_deposit: T::Balance,
    ) -> Result<(), DispatchError> {
      pallet_assets::Pallet::<T>::force_create(
        RawOrigin::Root.into(),
        asset_id,
        // make the pallet account id the owner, so only this pallet can handle the funds.
        T::Lookup::unlookup(Self::account_id()),
        true,
        1,
      )?;

      Ok(())
    }
  }
}
