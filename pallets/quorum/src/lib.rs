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
    traits::fungibles::{Inspect, Mutate, Transfer},
    PalletId,
  };
  use frame_system::pallet_prelude::*;
  use sp_runtime::traits::AccountIdConversion;
  use tidefi_primitives::{
    pallet::{AssetRegistryExt, QuorumExt, SecurityExt},
    AssetId, Balance, CurrencyId, Hash, Withdrawal, WithdrawalStatus,
  };

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config:
    frame_system::Config + pallet_assets::Config<AssetId = AssetId, Balance = Balance>
  {
    /// Events
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    /// Pallet ID
    #[pallet::constant]
    type QuorumPalletId: Get<PalletId>;

    /// Weights
    type WeightInfo: WeightInfo;

    /// Security traits
    type Security: SecurityExt<Self::AccountId, Self::BlockNumber>;

    /// Asset registry traits
    type AssetRegistry: AssetRegistryExt;

    /// Currency wrapr
    type CurrencyWrapr: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  /// Quorum status
  #[pallet::storage]
  #[pallet::getter(fn status)]
  pub(super) type QuorumStatus<T: Config> = StorageValue<_, bool, ValueQuery>;

  /// Quorum Account ID
  #[pallet::storage]
  #[pallet::getter(fn account_id)]
  pub type QuorumAccountId<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

  /// Mapping of pending Withdrawals
  #[pallet::storage]
  #[pallet::getter(fn withdrawals)]
  pub type Withdrawals<T: Config> =
    StorageMap<_, Blake2_128Concat, Hash, Withdrawal<T::AccountId, T::BlockNumber>>;

  /// Genesis configuration
  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    /// Quorum status
    pub enabled: bool,
    /// Quorum Account ID. Multisig is supported.
    /// This account will be able to confirm deposit / withdrawal on-chain.
    pub account: T::AccountId,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        // Quorum is enabled by default
        enabled: true,
        // We use pallet account ID by default,
        // but should always be set in the genesis config.
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
    StatusChanged { is_enabled: bool },
    /// Quorum account changed
    AccountChanged { account_id: T::AccountId },
    /// Quorum minted token to the account
    Minted {
      account_id: T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
    },
    /// Quorum burned token to the account
    Burned {
      request_id: Hash,
      account_id: T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
    },
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Asset is currently disabled or do not exist on chain
    AssetDisabled,
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

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Quorum have confirmation and make a new deposit for the asset.
    ///
    /// - `account_id`: Account Id to be deposited
    /// - `currency_id`: the currency ID
    /// - `mint_amount`: the amount to be deposited
    ///
    /// Emits `Minted` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::mint())]
    pub fn mint(
      origin: OriginFor<T>,
      account_id: T::AccountId,
      currency_id: CurrencyId,
      mint_amount: Balance,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the quorum/chain is not paused
      Self::ensure_not_paused()?;

      // 2. Make sure the request is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(Some(sender) == Self::account_id(), Error::<T>::AccessDenied);

      // 3. Make sure the currency_id exist and is enabled
      ensure!(
        T::AssetRegistry::is_currency_enabled(currency_id),
        Error::<T>::AssetDisabled
      );

      // 4. Mint the token
      T::CurrencyWrapr::mint_into(currency_id, &account_id, mint_amount)?;

      // 5. Send event on chain
      Self::deposit_event(Event::<T>::Minted {
        account_id,
        currency_id,
        amount: mint_amount,
      });

      Ok(().into())
    }

    /// Quorum have confirmation and process the withdrawal on-chain for the wrapped asset.
    ///
    /// This mean that the asset (example BTC) got processed by the quorum and the coin(s)
    /// have been sent in the BTC chain to the external address
    /// provided in the initial request.
    ///
    /// - `request_id`: Unique request ID.
    ///
    /// Emits `Minted` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::confirm_withdrawal())]
    pub fn confirm_withdrawal(
      origin: OriginFor<T>,
      request_id: Hash,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the quorum/chain is not paused
      Self::ensure_not_paused()?;

      // 2. Make sure the request is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(Some(sender) == Self::account_id(), Error::<T>::AccessDenied);

      // 3. Make sure the `request_id` exist
      Withdrawals::<T>::try_mutate_exists(request_id, |withdrawal| {
        match withdrawal {
          None => {
            return Err(Error::<T>::InvalidRequestId);
          }
          Some(withdrawal) => {
            // 3. Make sure the currency_id exist and is enabled
            ensure!(
              T::AssetRegistry::is_currency_enabled(withdrawal.asset_id),
              Error::<T>::AssetDisabled
            );

            // 4. Remove the token from the account
            T::CurrencyWrapr::burn_from(
              withdrawal.asset_id,
              &withdrawal.account_id,
              withdrawal.amount,
            )
            .map_err(|_| Error::<T>::BurnFailed)?;

            // 5. Emit the event on chain
            Self::deposit_event(Event::<T>::Burned {
              request_id,
              account_id: withdrawal.account_id.clone(),
              currency_id: withdrawal.asset_id,
              amount: withdrawal.amount,
            });
          }
        }

        // it deletes the item if mutated to a None.
        // FIXME: Should we update the status and keep a reference in our storage?
        *withdrawal = None;
        Ok(())
      })?;

      Ok(().into())
    }

    /// Change Quorum status.
    ///
    /// - `is_enabled`: Is the quorum enabled?
    ///
    /// Emits `StatusChanged` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::set_status())]
    pub fn set_status(origin: OriginFor<T>, quorum_enabled: bool) -> DispatchResultWithPostInfo {
      // 1. Make sure the request is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(Some(sender) == Self::account_id(), Error::<T>::AccessDenied);

      // 2. Update quorum status
      QuorumStatus::<T>::put(quorum_enabled);

      // 3. Emit event on chain
      Self::deposit_event(Event::<T>::StatusChanged {
        is_enabled: quorum_enabled,
      });
      Ok(().into())
    }

    /// Quorum change the account ID who can confirm withdrawal
    /// and mint new token on-chain.
    ///
    /// Make sure to have access to the `account_id` otherwise
    /// only `root` will be able to update the quorum account.
    ///
    /// - `new_account_id`: The new Quorum account id.
    ///
    /// Emits `AccountChanged` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::set_account_id())]
    pub fn set_account_id(
      origin: OriginFor<T>,
      new_account_id: T::AccountId,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the request is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(Some(sender) == Self::account_id(), Error::<T>::AccessDenied);

      // 2. Update quorum account id
      QuorumAccountId::<T>::put(new_account_id.clone());

      // 3. Emit event on chain
      Self::deposit_event(Event::<T>::AccountChanged {
        account_id: new_account_id,
      });

      Ok(().into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config> Pallet<T> {
    fn ensure_not_paused() -> Result<(), DispatchError> {
      if Self::is_quorum_enabled() {
        Ok(())
      } else {
        Err(Error::<T>::QuorumPaused.into())
      }
    }
  }

  // quorum extension exposed in other pallets
  impl<T: Config> QuorumExt<T::AccountId, T::BlockNumber> for Pallet<T> {
    /// Get quorum status
    fn is_quorum_enabled() -> bool {
      T::Security::is_chain_running() && Self::status()
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
