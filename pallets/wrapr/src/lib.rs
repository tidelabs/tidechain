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
  use frame_system::pallet_prelude::*;
  use sp_runtime::traits::{AccountIdConversion, StaticLookup};
  use tidefi_primitives::{AssetId, Balance};

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config:
    frame_system::Config
    + pallet_sudo::Config
    + pallet_assets::Config<AssetId = AssetId, Balance = Balance>
  {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    type Assets: Transfer<Self::AccountId> + Inspect<Self::AccountId> + Mutate<Self::AccountId>;
    /// The loan's module id, keep all collaterals of CDPs.
    #[pallet::constant]
    type PalletId: Get<PalletId>;
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

  /// Mapping of account addresses to deposit details
  /// Owner -> BalanceOf
  //FIXME: Not sure if its really needed?
  #[pallet::storage]
  #[pallet::getter(fn account_deposits)]
  pub type AccountDeposits<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AssetId,
    Blake2_128Concat,
    T::AccountId,
    T::Balance,
    ValueQuery,
  >;

  #[pallet::genesis_config]
  pub struct GenesisConfig {
    pub quorum_enabled: bool,
  }

  #[cfg(feature = "std")]
  impl Default for GenesisConfig {
    fn default() -> Self {
      Self {
        quorum_enabled: true,
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig {
    fn build(&self) {
      QuorumStatus::<T>::put(self.quorum_enabled);
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Quorum status changed
    /// [is_enabled]
    QuorumStatusChanged(bool),
    /// Event emitted when assets are deposited
    /// [sender, asset_id, amount]
    Deposited(T::AccountId, T::AssetId, T::Balance),
    /// Event emitted when widthdraw is requested. The Quorum will process the wrapped
    /// refund (example BTC) and will send confirmation once completed.
    /// [sender, asset_id, amount]
    Withdrawal(T::AccountId, T::AssetId, T::Balance),
    /// Event emitted when widthdraw is completed
    /// [sender, asset_id, amount]
    Burned(T::AccountId, T::AssetId, T::Balance),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// The Quorum is paused. Try again later.
    QuorumPaused,
    /// Unknown Asset.
    UnknownAsset,
    /// No Funds available for this Asset Id.
    NoFunds,
    /// Unknown Error.
    UnknownError,
  }

  // Dispatchable functions allows users to interact with the pallet and invoke state changes.
  // These functions materialize as "extrinsics", which are often compared to transactions.
  // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
  #[pallet::call]
  impl<T: Config> Pallet<T> {
    // Change quorum status
    #[pallet::weight(10000)]
    pub fn set_quorum_status(
      origin: OriginFor<T>,
      quorum_enabled: bool,
    ) -> DispatchResultWithPostInfo {
      // make sure it's the quprum
      ensure_root(origin)?;
      // update quorum
      QuorumStatus::<T>::put(quorum_enabled);
      // no payment required
      Ok(Pays::No.into())
    }

    /// AccountID request withdrawal.
    /// This will dispatch an Event on the chain and the Quprum should listen to process the job
    /// and send the confirmation once done.
    #[pallet::weight(10000)]
    pub fn request_withdrawal(
      origin: OriginFor<T>,
      asset_id: T::AssetId,
      amount: T::Balance,
    ) -> DispatchResultWithPostInfo {
      let who = ensure_signed(origin)?;
      // make sure the account have the fund to save some time
      // to the quorum
      match pallet_assets::Pallet::<T>::can_withdraw(asset_id, &who, amount) {
        WithdrawConsequence::Success => {
          // send event to the chain
          Self::deposit_event(Event::<T>::Withdrawal(who, asset_id, amount));
          // ok
          Ok(Pays::No.into())
        }
        WithdrawConsequence::NoFunds => Err(Error::<T>::NoFunds.into()),
        WithdrawConsequence::UnknownAsset => Err(Error::<T>::UnknownAsset.into()),
        _ => Err(Error::<T>::UnknownError.into()),
      }
    }

    /// Quorum have confirmation and make a new deposit for the asset.
    ///
    /// If the asset id do not exist, it get created.
    ///
    /// - `account_id`: Account Id to be deposited.
    /// - `asset_id`: the asset to be deposited.
    /// - `mint_amount`: the amount to be deposited.
    #[pallet::weight(10000)]
    pub fn quorum_do_mint(
      origin: OriginFor<T>,
      account_id: T::AccountId,
      asset_id: T::AssetId,
      mint_amount: T::Balance,
    ) -> DispatchResultWithPostInfo {
      // make sure it's the quorum
      ensure_root(origin.clone())?;
      // make sure the quorum is not paused
      Self::ensure_not_paused()?;

      // make sure the currency exists, the pallet failed if already exist but we don't really care.
      // FIXME: Maybe we could check if the failed is because of the asset already exist.
      // otherwise we should failed here
      let _error = pallet_assets::Pallet::<T>::force_create(
        origin.clone(),
        asset_id,
        // FIXME:
        // the owner is the pallet ID and not the quorum
        T::Lookup::unlookup(Self::account_id()),
        true,
        0,
      );

      // mint the token
      pallet_assets::Pallet::<T>::mint(
        origin,
        asset_id,
        T::Lookup::unlookup(account_id.clone()),
        mint_amount,
      )?;

      // Keep the deposit record for the account / asset id
      AccountDeposits::<T>::try_mutate(asset_id, &account_id, |deposits| -> DispatchResult {
        *deposits += mint_amount;
        Ok(())
      })?;

      // send event to the chain
      Self::deposit_event(Event::<T>::Deposited(account_id, asset_id, mint_amount));

      Ok(Pays::No.into())
    }

    /// Quorum have confirmation and make a new burn (widthdraw).
    ///
    /// - `account_id`: Account Id to remove the funds.
    /// - `asset_id`: the asset to remove the funds.
    /// - `burn_amount`: the amount to remove for this asset.
    #[pallet::weight(10000)]
    pub fn quorum_do_burn(
      origin: OriginFor<T>,
      account_id: T::AccountId,
      asset_id: T::AssetId,
      burn_amount: T::Balance,
    ) -> DispatchResultWithPostInfo {
      // make sure it's the quorum
      ensure_root(origin.clone())?;
      // make sure the quorum is not paused
      Self::ensure_not_paused()?;

      // Transfer the asset fund to the Wrapr Account (using pallet_assets)
      pallet_assets::Pallet::<T>::burn(
        origin,
        asset_id,
        T::Lookup::unlookup(account_id.clone()),
        burn_amount,
      )?;

      // Keep the deposit record for the account / asset id
      AccountDeposits::<T>::try_mutate(asset_id, &account_id, |deposits| -> DispatchResult {
        *deposits -= burn_amount;
        Ok(())
      })?;

      // send event to the chain
      Self::deposit_event(Event::<T>::Burned(account_id, asset_id, burn_amount));

      Ok(Pays::No.into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config + pallet_assets::Config<AssetId = AssetId, Balance = Balance>> Pallet<T> {
    pub fn account_id() -> T::AccountId {
      T::PalletId::get().into_account()
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
