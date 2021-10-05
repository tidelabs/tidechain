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
    inherent::Vec,
    pallet_prelude::*,
    traits::{
      fungibles::{Inspect, Mutate, Transfer},
      tokens::WithdrawConsequence,
    },
    PalletId,
  };
  use frame_system::{pallet_prelude::*, RawOrigin};
  use sp_runtime::traits::{AccountIdConversion, StaticLookup};
  use tidefi_primitives::{
    pallet::{QuorumExt, SecurityExt},
    AssetId, Balance, CurrencyId, Hash, RequestId, Trade, TradeStatus, Withdrawal,
    WithdrawalStatus,
  };

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

    /// Security traits
    type Security: SecurityExt<Self::AccountId, Self::BlockNumber>;

    type CurrencyWrapr: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
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
  pub type QuorumAccountId<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

  /// Mapping of pending Withdrawals
  #[pallet::storage]
  #[pallet::getter(fn withdrawals)]
  pub type Withdrawals<T: Config> =
    StorageMap<_, Blake2_128Concat, Hash, Withdrawal<T::AccountId, T::BlockNumber>>;

  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    pub enabled: bool,
    pub account: T::AccountId,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        enabled: true,
        account: T::QuorumPalletId::get().into_account(),
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      QuorumStatus::<T>::put(self.enabled);
      QuorumAccountId::<T>::put(self.account.clone());
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Quorum status changed
    /// [is_enabled]
    StatusChanged(bool),
    /// Quorum account changed
    /// [account_id]
    AccountChanged(T::AccountId),
    /// Quorum minted token to the account
    /// [sender, asset_id, amount]
    Minted(T::AccountId, CurrencyId, Balance),
    /// Quorum burned token to the account
    /// [sender, asset_id, amount]
    Burned(T::AccountId, CurrencyId, Balance),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// The Quorum is paused. Try again later.
    QuorumPaused,
    /// The access to the Quorum pallet is not allowed for this account ID.
    AccessDenied,
    /// Invalid request ID.
    InvalidRequestId,
    /// There is a conflict in the request.
    Conflict,
    /// Unable to burn token.
    BurnFailed,
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
    #[pallet::weight(<T as pallet::Config>::WeightInfo::mint())]
    pub fn mint(
      origin: OriginFor<T>,
      account_id: T::AccountId,
      asset_id: CurrencyId,
      mint_amount: Balance,
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
      if let CurrencyId::Wrapped(asset) = asset_id {
        let _force_create = pallet_assets::Pallet::<T>::force_create(
          RawOrigin::Root.into(),
          asset,
          // make the pallet account id the owner, so only this pallet can handle the funds.
          T::Lookup::unlookup(Self::account_id()),
          true,
          1,
        );
      }

      // mint the token
      T::CurrencyWrapr::mint_into(asset_id, &account_id, mint_amount)?;

      // send event to the chain
      Self::deposit_event(Event::<T>::Minted(account_id, asset_id, mint_amount));

      Ok(().into())
    }

    /// Quorum have confirmation and make a new burn (widthdraw).
    ///
    /// - `request_id`: Request ID.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::confirm_withdrawal())]
    pub fn confirm_withdrawal(
      origin: OriginFor<T>,
      request_id: Hash,
    ) -> DispatchResultWithPostInfo {
      // make sure the quorum is not paused
      Self::ensure_not_paused()?;
      // make sure it's the quorum
      let sender = ensure_signed(origin)?;
      ensure!(
        sender == Self::quorum_account_id(),
        Error::<T>::AccessDenied
      );

      Withdrawals::<T>::try_mutate_exists(request_id, |withdrawal| {
        match withdrawal {
          None => {
            return Err(Error::<T>::InvalidRequestId);
          }
          Some(withdrawal) => {
            // remove the token from the account
            T::CurrencyWrapr::burn_from(
              withdrawal.asset_id,
              &withdrawal.account_id,
              withdrawal.amount,
            )
            .map_err(|_| Error::<T>::BurnFailed)?;

            Self::deposit_event(Event::<T>::Burned(
              withdrawal.account_id.clone(),
              withdrawal.asset_id,
              withdrawal.amount,
            ));
          }
        }
        // it deletes the item if mutated to a None.
        *withdrawal = None;
        Ok(())
      })?;

      Ok(().into())
    }

    /// Quorum change status.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::set_status())]
    pub fn set_status(origin: OriginFor<T>, quorum_enabled: bool) -> DispatchResultWithPostInfo {
      // make sure it's the quorum
      let sender = ensure_signed(origin)?;
      ensure!(
        sender == Self::quorum_account_id(),
        Error::<T>::AccessDenied
      );
      // update quorum
      QuorumStatus::<T>::put(quorum_enabled);
      // emit event
      Self::deposit_event(Event::<T>::StatusChanged(quorum_enabled));
      Ok(().into())
    }

    /// Quorum change account ID.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::set_status())]
    pub fn set_account_id(
      origin: OriginFor<T>,
      new_quorum: T::AccountId,
    ) -> DispatchResultWithPostInfo {
      // make sure it's the quorum
      let sender = ensure_signed(origin)?;
      ensure!(
        sender == Self::quorum_account_id(),
        Error::<T>::AccessDenied
      );

      // update quorum
      QuorumAccountId::<T>::put(new_quorum.clone());

      // emit event
      Self::deposit_event(Event::<T>::AccountChanged(new_quorum));

      Ok(().into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config> Pallet<T> {
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

  impl<T: Config> QuorumExt<T::AccountId, T::BlockNumber> for Pallet<T> {
    /// Get quorum status
    fn is_quorum_enabled() -> bool {
      T::Security::is_chain_running() && Self::is_quorum_enabled()
    }

    /// Add new withdrawal in queue
    fn add_new_withdrawal_in_queue(
      account_id: T::AccountId,
      asset_id: CurrencyId,
      amount: Balance,
      external_address: Vec<u8>,
    ) -> (Hash, Withdrawal<T::AccountId, T::BlockNumber>) {
      let request_id = T::Security::get_unique_id(account_id.clone());
      let withdrawal = Withdrawal {
        account_id,
        amount,
        asset_id,
        external_address,
        status: WithdrawalStatus::Pending,
        block_number: <frame_system::Pallet<T>>::block_number(),
      };

      // insert in our queue
      Withdrawals::<T>::insert(request_id, withdrawal.clone());

      // return values
      (request_id, withdrawal)
    }
  }
}
