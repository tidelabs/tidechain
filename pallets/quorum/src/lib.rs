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
  use sp_runtime::traits::{AccountIdConversion, StaticLookup};
  use tidefi_primitives::{
    pallet::{QuorumExt, WraprExt},
    AssetId, Balance, RequestId, Stake, Trade, TradeStatus, Withdrawal, WithdrawalStatus,
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

  /// Requests counter
  #[pallet::storage]
  pub type RequestCounter<T: Config> = StorageValue<_, RequestId, ValueQuery>;

  /// Mapping of pending Withdrawals
  #[pallet::storage]
  #[pallet::getter(fn withdrawals)]
  pub type Withdrawals<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    RequestId,
    Withdrawal<T::AccountId, T::AssetId, T::Balance, T::BlockNumber>,
  >;

  /// Mapping of pending Trades
  #[pallet::storage]
  #[pallet::getter(fn trades)]
  pub type Trades<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    RequestId,
    Trade<T::AccountId, T::AssetId, T::Balance, T::BlockNumber>,
  >;

  /// Mapping of pending Stakes
  #[pallet::storage]
  #[pallet::getter(fn stakes)]
  pub type Stakes<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    RequestId,
    Stake<T::AccountId, T::AssetId, T::Balance, T::BlockNumber>,
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
    /// Invalid withdrawal ID.
    InvalidRequestId,
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
      let _force_create = pallet_assets::Pallet::<T>::force_create(
        RawOrigin::Root.into(),
        asset_id,
        // make the pallet account id the owner, so only this pallet can handle the funds.
        T::Lookup::unlookup(Self::account_id()),
        true,
        1,
      );

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
    /// - `withdrawal_id`: Withdrawal ID.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::burn())]
    pub fn burn(origin: OriginFor<T>, withdrawal_id: RequestId) -> DispatchResultWithPostInfo {
      // make sure the quorum is not paused
      Self::ensure_not_paused()?;
      // make sure it's the quorum
      let sender = ensure_signed(origin)?;
      ensure!(
        sender == Self::quorum_account_id(),
        Error::<T>::AccessDenied
      );

      Withdrawals::<T>::try_mutate_exists(withdrawal_id, |withdrawal| {
        match withdrawal {
          None => {
            return Err(Error::<T>::InvalidRequestId);
          }
          Some(withdrawal) => {
            // remove the token from the account
            pallet_assets::Pallet::<T>::burn(
              RawOrigin::Signed(Self::account_id()).into(),
              withdrawal.asset_id,
              T::Lookup::unlookup(withdrawal.account_id.clone()),
              withdrawal.amount,
            )
            .map_err(|_| Error::<T>::BurnFailed)?;

            // FIXME: we can probably remove this and only use the
            // event emitted by the Assets pallet
            // emit the burned event
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

  impl<T: Config> QuorumExt<T::AccountId, T::AssetId, T::Balance, T::BlockNumber> for Pallet<T> {
    /// Get quorum status
    fn is_quorum_enabled() -> bool {
      Self::is_quorum_enabled()
    }

    /// Update quprum status
    fn set_quorum_status(quorum_enabled: bool) {
      // update quorum
      QuorumStatus::<T>::put(quorum_enabled);
      // emit event
      Self::deposit_event(Event::<T>::QuorumStatusChanged(quorum_enabled));
    }

    /// Add new withdrawal in queue
    fn add_new_withdrawal_in_queue(
      account_id: T::AccountId,
      asset_id: T::AssetId,
      amount: T::Balance,
      external_address: Vec<u8>,
    ) -> (
      RequestId,
      Withdrawal<T::AccountId, T::AssetId, T::Balance, T::BlockNumber>,
    ) {
      let request_id = <RequestCounter<T>>::get().wrapping_add(1);
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

    fn add_new_trade_in_queue(
      account_id: T::AccountId,
      asset_id_from: T::AssetId,
      amount_from: T::Balance,
      asset_id_to: T::AssetId,
      amount_to: T::Balance,
    ) -> (
      RequestId,
      Trade<T::AccountId, T::AssetId, T::Balance, T::BlockNumber>,
    ) {
      let request_id = <RequestCounter<T>>::get().wrapping_add(1);
      let trade = Trade {
        account_id,
        token_from: asset_id_from,
        token_to: asset_id_to,
        amount_from,
        amount_to,
        status: TradeStatus::Pending,
        block_number: <frame_system::Pallet<T>>::block_number(),
      };

      // insert in our queue
      Trades::<T>::insert(request_id, trade.clone());

      // return values
      (request_id, trade)
    }

    fn add_new_stake_in_queue(
      account_id: T::AccountId,
      asset_id: T::AssetId,
      amount: T::Balance,
      duration: u32,
    ) -> (
      RequestId,
      Stake<T::AccountId, T::AssetId, T::Balance, T::BlockNumber>,
    ) {
      let request_id = <RequestCounter<T>>::get().wrapping_add(1);
      let stake = Stake {
        account_id,
        asset_id,
        amount,
        duration,
        block_number: <frame_system::Pallet<T>>::block_number(),
      };

      // insert in our queue
      Stakes::<T>::insert(request_id, stake.clone());

      // return values
      (request_id, stake)
    }
  }
}
