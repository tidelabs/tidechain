#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

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
    traits::tokens::fungibles::{Inspect, Mutate, Transfer},
    PalletId,
  };
  use frame_system::pallet_prelude::*;
  use sp_runtime::{traits::AccountIdConversion, ArithmeticError};
  use tidefi_primitives::{pallet::AssetRegistryExt, Balance, BalanceInfo, CurrencyId, Stake};

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config: frame_system::Config {
    /// Events
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    /// Pallet ID
    #[pallet::constant]
    type StakePalletId: Get<PalletId>;

    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;

    /// Basis of period
    type PeriodBasis: Get<BlockNumberFor<Self>>;

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

  /// Staking pool
  #[pallet::storage]
  #[pallet::getter(fn staking_pool)]
  pub type StakingPool<T: Config> = StorageMap<_, Blake2_128Concat, CurrencyId, Balance>;

  /// Account staking
  #[pallet::storage]
  #[pallet::getter(fn account_stakes)]
  pub type AccountStakes<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    Blake2_128Concat,
    CurrencyId,
    Vec<Stake<Balance>>,
    ValueQuery,
  >;

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// The assets get staked successfully
    Staked {
      account_id: T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
    },
    /// The assets get unstaked successfully
    Unstaked {
      account_id: T::AccountId,
      currency_id: CurrencyId,
      initial_amount: Balance,
      final_amount: Balance,
    },
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {}

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Stake currency
    ///
    /// - `currency_id`: The currency to stake
    /// - `amount`: The amount to stake
    /// - `duration`: The duration is in numbers of blocks. (blocks are ~3seconds)
    ///
    /// Emits `Staked` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::stake())]
    pub fn stake(
      origin: OriginFor<T>,
      currency_id: CurrencyId,
      amount: Balance,
      duration: u32,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the transaction is signed
      let account_id = ensure_signed(origin)?;

      // 2. Transfer the funds into the staking pool
      T::CurrencyWrapr::transfer(currency_id, &account_id, &Self::account_id(), amount, true)?;

      // 3. Insert the new staking
      AccountStakes::<T>::mutate(account_id.clone(), currency_id, |stake| {
        stake.push(Stake {
          initial_balance: amount,
          principal: amount,
          duration,
        });
      });

      // 4. Update our `StakingPool` storage
      StakingPool::<T>::try_mutate(currency_id, |balance| -> DispatchResult {
        if let Some(b) = balance {
          *balance = Some(b.checked_add(amount).ok_or(ArithmeticError::Overflow)?);
        } else {
          *balance = Some(amount)
        }
        Ok(())
      })?;

      // 5. Emit event on chain
      Self::deposit_event(Event::<T>::Staked{ account_id, currency_id, amount});

      Ok(().into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
      <T as pallet::Config>::StakePalletId::get().into_account()
    }

    // Get all stakes for the account, serialized for quick RPC call
    pub fn get_account_stakes(account_id: &T::AccountId) -> Vec<(CurrencyId, Stake<BalanceInfo>)> {
      let mut final_stakes = Vec::new();

      let all_stakes: Vec<(CurrencyId, Vec<Stake<Balance>>)> =
        AccountStakes::<T>::iter_prefix(account_id).collect();

      // we need to re-organize as our storage use a unique AccountId / CurrencyId key
      for (currency_id, currency_stakes) in all_stakes {
        for currency_stake in currency_stakes {
          final_stakes.push((
            currency_id,
            Stake {
              principal: BalanceInfo {
                amount: currency_stake.principal,
              },
              initial_balance: BalanceInfo {
                amount: currency_stake.initial_balance,
              },
              duration: currency_stake.duration,
            },
          ));
        }
      }

      final_stakes
    }
  }
}
