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
  #[cfg(feature = "std")]
  use sp_runtime::traits::AccountIdConversion;
  use sp_std::vec;
  use tidefi_primitives::{
    pallet::{AssetRegistryExt, QuorumExt, SecurityExt},
    AssetId, Balance, ComplianceLevel, CurrencyId, Hash, WatchList, WatchListAction, Withdrawal,
    WithdrawalStatus,
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

    /// Tidechain currency wrapper
    type CurrencyTidefi: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
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

  /// Quorum public keys for all chains
  #[pallet::storage]
  #[pallet::getter(fn public_keys)]
  pub type PublicKeys<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    Blake2_128Concat,
    AssetId,
    Vec<u8>,
    ValueQuery,
  >;

  /// Set of active transaction to watch
  #[pallet::storage]
  #[pallet::getter(fn account_watch_list)]
  pub type AccountWatchList<T: Config> =
    StorageMap<_, Blake2_128Concat, T::AccountId, Vec<WatchList<T::BlockNumber>>>;

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
      transaction_id: Vec<u8>,
      compliance_level: ComplianceLevel,
    },
    /// A new transaction has been added to the watch list
    WatchTransactionAdded {
      account_id: T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
      compliance_level: ComplianceLevel,
      transaction_id: Vec<u8>,
      watch_action: WatchListAction,
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
      transaction_id: Vec<u8>,
      compliance_level: ComplianceLevel,
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

      // 4. Add `Amber` and `Red` to watch list
      if compliance_level.clone() == ComplianceLevel::Amber
        || compliance_level.clone() == ComplianceLevel::Red
      {
        Self::add_account_watch_list(
          &account_id,
          currency_id,
          mint_amount,
          compliance_level.clone(),
          transaction_id.clone(),
          WatchListAction::Mint,
        );
      }

      // 5. Mint `Green` and `Amber`
      if compliance_level.clone() == ComplianceLevel::Green
        || compliance_level.clone() == ComplianceLevel::Amber
      {
        T::CurrencyTidefi::mint_into(currency_id, &account_id, mint_amount)?;
        Self::deposit_event(Event::<T>::Minted {
          account_id,
          currency_id,
          amount: mint_amount,
          transaction_id,
          compliance_level,
        });
      }

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
            T::CurrencyTidefi::burn_from(
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

      // don't take tx fees on success
      Ok(Pays::No.into())
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

      // don't take tx fees on success
      Ok(Pays::No.into())
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

      // don't take tx fees on success
      Ok(Pays::No.into())
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

    fn add_account_watch_list(
      account_id: &T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
      compliance_level: ComplianceLevel,
      transaction_id: Vec<u8>,
      watch_action: WatchListAction,
    ) {
      let block_number = <frame_system::Pallet<T>>::block_number();
      let watch_list = WatchList {
        amount,
        block_number,
        compliance_level: compliance_level.clone(),
        currency_id,
        watch_action: watch_action.clone(),
        transaction_id: transaction_id.clone(),
      };

      AccountWatchList::<T>::mutate_exists(
        account_id,
        |account_watch_list| match account_watch_list {
          Some(current_watch_list) => current_watch_list.push(watch_list),
          None => AccountWatchList::<T>::insert(account_id, vec![watch_list]),
        },
      );

      Self::deposit_event(Event::<T>::WatchTransactionAdded {
        account_id: account_id.clone(),
        currency_id,
        amount,
        compliance_level,
        watch_action,
        transaction_id,
      });
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
