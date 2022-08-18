// Copyright 2021-2022 Semantic Network Ltd.
// This file is part of Tidechain.

// Tidechain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tidechain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tidechain.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
  use frame_support::pallet_prelude::*;
  use frame_system::pallet_prelude::*;
  use sha2::{Digest, Sha256};
  use sp_core::{H256, U256};
  use sp_runtime::traits::Saturating;
  use tidefi_primitives::{pallet::SecurityExt, Hash, StatusCode};

  /// Security configuration
  #[pallet::config]
  pub trait Config: frame_system::Config {
    /// Events
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  /// Integer increment-only counter
  #[pallet::storage]
  pub type Nonce<T: Config> = StorageValue<_, U256, ValueQuery>;

  /// Chain status, currently support maintenance mode/running state
  #[pallet::storage]
  #[pallet::getter(fn status)]
  pub type ChainStatus<T: Config> = StorageValue<_, StatusCode, ValueQuery>;

  /// Current block number (when the chain is paused, the block count is not incremented,
  /// so we don't give rewards on staking)
  #[pallet::storage]
  #[pallet::getter(fn current_block_number)]
  pub type CurrentBlockCount<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

  /// Genesis config
  #[pallet::genesis_config]
  pub struct GenesisConfig {
    /// Chain status
    pub status: StatusCode,
  }

  #[cfg(feature = "std")]
  impl Default for GenesisConfig {
    fn default() -> Self {
      Self {
        // default to running state
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
    StatusChanged(StatusCode),
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

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Change chain status.
    ///
    /// This should be used only as emergency as you can disable Oracle or
    /// Quorum independently. You can also disable a currency if it's a known issue with
    /// a bridge. All transactions will be rejected, even from the Oracle
    /// and the Quorum for security reason.
    ///
    /// This will also stop the staking blocks increment, so even if the Validators
    /// are still building blocks, they are not counted for the rewards.
    ///
    /// - `status_code`: New chain `StatusCode`
    ///
    /// Emits `StatusChanged` event when successful.
    #[pallet::weight(0)]
    pub fn set_status(origin: OriginFor<T>, status_code: StatusCode) -> DispatchResultWithPostInfo {
      ensure_root(origin)?;
      <ChainStatus<T>>::set(status_code.clone());
      Self::deposit_event(Event::StatusChanged(status_code));
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
      hasher.update(id.encode());
      hasher.update(Self::get_nonce().encode());
      // supplement with prev block hash to prevent replays
      // even if the `Nonce` is reset (i.e. purge-chain)
      hasher.update(frame_system::Pallet::<T>::parent_hash());
      let mut result = [0; 32];
      result.copy_from_slice(&hasher.finalize()[..]);
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
