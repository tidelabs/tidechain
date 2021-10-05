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
  use frame_support::pallet_prelude::*;
  use frame_system::pallet_prelude::*;
  use sha2::{Digest, Sha256};
  use sp_core::{H256, U256};
  use sp_runtime::traits::Saturating;
  use tidefi_primitives::{
    pallet::SecurityExt, AccountId, AssetId, Balance, BlockNumber, Hash, StatusCode,
  };

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config:
    frame_system::Config + pallet_assets::Config<AssetId = AssetId, Balance = Balance>
  {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  /// Integer increment-only counter, used to prevent collisions when generating identifiers
  #[pallet::storage]
  pub type Nonce<T: Config> = StorageValue<_, U256, ValueQuery>;

  /// Integer - enum defining the current state of the chain
  #[pallet::storage]
  #[pallet::getter(fn status)]
  pub type ChainStatus<T: Config> = StorageValue<_, StatusCode, ValueQuery>;

  #[pallet::storage]
  #[pallet::getter(fn current_block_number)]
  pub type CurrentBlockCount<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

  #[pallet::genesis_config]
  pub struct GenesisConfig {
    pub status: StatusCode,
  }

  #[cfg(feature = "std")]
  impl Default for GenesisConfig {
    fn default() -> Self {
      Self {
        status: StatusCode::Running,
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig {
    fn build(&self) {
      <ChainStatus<T>>::set(self.status.clone());
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Processing new block
    UpdateCurrentBlock(T::BlockNumber),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Chain is in maintenance mode
    ChainMaintenanceMode,
  }

  #[pallet::hooks]
  impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
    fn on_initialize(_current_block: T::BlockNumber) -> Weight {
      // updated ONLY when the chain status is running
      if Self::status() == StatusCode::Running {
        let height = <CurrentBlockCount<T>>::mutate(|n| {
          *n = n.saturating_add(1u32.into());
          *n
        });
        Self::deposit_event(Event::UpdateCurrentBlock(height));
      }
      0
    }
  }

  // Dispatchable functions allows users to interact with the pallet and invoke state changes.
  // These functions materialize as "extrinsics", which are often compared to transactions.
  // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Set the chain status code.
    #[pallet::weight(0)]
    pub fn set_status(origin: OriginFor<T>, status_code: StatusCode) -> DispatchResultWithPostInfo {
      ensure_root(origin)?;
      <ChainStatus<T>>::set(status_code);
      Ok(().into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config> Pallet<T> {
    /// Increment and return the `Nonce`.
    fn get_nonce() -> U256 {
      <Nonce<T>>::mutate(|n| {
        let (res, _) = (*n).overflowing_add(U256::one());
        *n = res;
        *n
      })
    }

    /// Generates a 256-bit unique hash from an `AccountId` and the
    /// internal (auto-incrementing) `Nonce` to prevent replay attacks.
    fn get_next_id(id: &T::AccountId) -> H256 {
      let mut hasher = Sha256::default();
      hasher.input(id.encode());
      hasher.input(Self::get_nonce().encode());
      // supplement with prev block hash to prevent replays
      // even if the `Nonce` is reset (i.e. purge-chain)
      hasher.input(frame_system::Pallet::<T>::parent_hash());
      let mut result = [0; 32];
      result.copy_from_slice(&hasher.result()[..]);
      H256(result)
    }
  }

  impl<T: Config> SecurityExt<T::AccountId, T::BlockNumber> for Pallet<T> {
    fn is_chain_running() -> bool {
      Self::status() == StatusCode::Running
    }
    fn get_current_block_count() -> T::BlockNumber {
      Self::current_block_number()
    }
    fn get_unique_id(account_id: T::AccountId) -> Hash {
      Self::get_next_id(&account_id)
    }
  }
}
