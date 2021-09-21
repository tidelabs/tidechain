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
  use frame_support::{
    pallet_prelude::*,
    traits::tokens::{
      fungibles::{Inspect, Mutate, Transfer},
      WithdrawConsequence,
    },
    transactional, PalletId,
  };
  use frame_system::{pallet_prelude::*, Origin, RawOrigin};
  use sp_runtime::{
    traits::{AccountIdConversion, StaticLookup},
    ArithmeticError,
  };
  use tidefi_primitives::{AssetId, Balance};

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config:
    frame_system::Config + pallet_assets::Config<AssetId = AssetId, Balance = Balance>
  {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    #[pallet::constant]
    type QuorumPalletId: Get<PalletId>;
    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  /// Quorum is enabled
  #[pallet::storage]
  #[pallet::getter(fn is_quorum_enabled)]
  pub(super) type QuorumStatus<T: Config> = StorageValue<_, bool, ValueQuery>;

  /// Quorum Account ID
  #[pallet::storage]
  #[pallet::getter(fn quorum_account_id)]
  pub type QuorumAccountID<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

  /// Mapping of pending Withdrawals (AssetId, AccountId)
  #[pallet::storage]
  #[pallet::getter(fn withdrawals)]
  pub type Withdrawals<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AssetId,
    Blake2_128Concat,
    T::AccountId,
    T::Balance,
    ValueQuery,
  >;

  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    pub quorum_enabled: bool,
    pub quorum_account: T::AccountId,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        quorum_enabled: true,
        quorum_account: T::QuorumPalletId::get().into_account(),
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      QuorumStatus::<T>::put(self.quorum_enabled);
      QuorumAccountID::<T>::put(self.quorum_account.clone());
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Quorum status changed
    /// [is_enabled]
    QuorumStatusChanged(bool),
    /// Quorum account changed
    /// [account_id]
    QuorumAccountChanged(T::AccountId),
    /// Quorum minted token to the account
    /// [sender, asset_id, amount]
    Minted(T::AccountId, T::AssetId, T::Balance),
    /// Quorum burned token to the account
    /// [sender, asset_id, amount]
    Burned(T::AccountId, T::AssetId, T::Balance),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// The Quorum is paused. Try again later.
    QuorumPaused,
    /// The access to the Quorum pallet is not allowed for this account ID.
    AccessDenied,
  }

  // Dispatchable functions allows users to interact with the pallet and invoke state changes.
  // These functions materialize as "extrinsics", which are often compared to transactions.
  // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Quorum have confirmation and make a new deposit for the asset.
    ///
    /// If the asset id do not exist, it get created.
    ///
    /// - `account_id`: Account Id to be deposited.
    /// - `asset_id`: the asset to be deposited.
    /// - `mint_amount`: the amount to be deposited.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::burn())]
    pub fn mint(
      origin: OriginFor<T>,
      account_id: T::AccountId,
      asset_id: T::AssetId,
      mint_amount: T::Balance,
    ) -> DispatchResultWithPostInfo {
      // make sure it's the quorum
      let sender = ensure_signed(origin)?;
      ensure!(
        sender == Self::quorum_account_id(),
        Error::<T>::AccessDenied
      );

      // make sure the currency exists, the pallet failed if already exist but we don't really care.
      // FIXME: Maybe we could check if the failed is because of the asset already exist.
      // otherwise we should failed here
      if let Err(dispatch_error) = pallet_assets::Pallet::<T>::force_create(
        RawOrigin::Root.into(),
        asset_id,
        T::Lookup::unlookup(Self::account_id()),
        true,
        1,
      ) {
        if let DispatchError::Module { .. } = dispatch_error {
          //println!("Error code: {}", error);
        }
      }

      // mint the token
      pallet_assets::Pallet::<T>::mint(
        RawOrigin::Signed(Self::account_id()).into(),
        asset_id,
        T::Lookup::unlookup(account_id.clone()),
        mint_amount,
      )?;

      // send event to the chain
      Self::deposit_event(Event::<T>::Minted(account_id, asset_id, mint_amount));

      Ok(Pays::No.into())
    }

    /// Quorum have confirmation and make a new burn (widthdraw).
    ///
    /// - `account_id`: Account Id to remove the funds.
    /// - `asset_id`: the asset to remove the funds.
    /// - `burn_amount`: the amount to remove for this asset.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::burn())]
    pub fn burn(
      origin: OriginFor<T>,
      account_id: T::AccountId,
      asset_id: T::AssetId,
      burn_amount: T::Balance,
    ) -> DispatchResultWithPostInfo {
      // make sure the quorum is not paused
      Self::ensure_not_paused()?;

      // make sure it's the quorum
      let sender = ensure_signed(origin)?;
      ensure!(
        sender == Self::quorum_account_id(),
        Error::<T>::AccessDenied
      );

      // Transfer the asset fund to the Wrapr Account (using pallet_assets)
      pallet_assets::Pallet::<T>::burn(
        RawOrigin::Signed(Self::account_id()).into(),
        asset_id,
        T::Lookup::unlookup(account_id.clone()),
        burn_amount,
      )?;

      // send event to the chain
      Self::deposit_event(Event::<T>::Burned(
        account_id.clone(),
        asset_id,
        burn_amount,
      ));

      /*
      // Remove the pending withdrawal
      Withdrawals::<T>::try_mutate_exists(
        asset_id,
        account_id,
        |current_value: &mut Option<T::Balance>| -> DispatchResult {
          let new_value = current_value.unwrap_or_default().checked_add(burn_amount);
          // we don't want to have our queue manager to failed
          if let Some(new_value) = new_value {
            // if we have a positive balance, we keep it in the queue
            if new_value <= 0 {
              *current_value = None;
            } else {
              *current_value = Some(new_value);
            }
          }
          Ok(())
        },
      )?;
      */
      Ok(Pays::No.into())
    }

    /// Quorum change status.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::set_status())]
    pub fn set_status(origin: OriginFor<T>, quorum_enabled: bool) -> DispatchResultWithPostInfo {
      // make sure the quorum is not paused
      Self::ensure_not_paused()?;

      // make sure it's the quorum
      let sender = ensure_signed(origin)?;
      ensure!(
        sender == Self::quorum_account_id(),
        Error::<T>::AccessDenied
      );

      // update quorum
      QuorumStatus::<T>::put(quorum_enabled);

      // emit event
      Self::deposit_event(Event::<T>::QuorumStatusChanged(quorum_enabled));

      // no payment required
      Ok(Pays::No.into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config + pallet_assets::Config<AssetId = AssetId, Balance = Balance>> Pallet<T> {
    pub fn account_id() -> T::AccountId {
      T::QuorumPalletId::get().into_account()
    }

    fn ensure_not_paused() -> Result<(), DispatchError> {
      if Self::is_quorum_enabled() {
        Ok(())
      } else {
        Err(Error::<T>::QuorumPaused.into())
      }
    }
  }
}
