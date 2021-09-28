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
  use tidefi_primitives::{Balance, BalanceInfo, CurrencyId, Stake};

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    type Assets: Transfer<Self::AccountId> + Inspect<Self::AccountId> + Mutate<Self::AccountId>;
    #[pallet::constant]
    type PalletId: Get<PalletId>;
    /// Quorum currency.
    type CurrencyWrapr: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;
    /// Basis of period.
    //#[pallet::constant]
    type PeriodBasis: Get<BlockNumberFor<Self>>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  #[pallet::storage]
  #[pallet::getter(fn staking_pool)]
  pub type StakingPool<T: Config> = StorageMap<_, Blake2_128Concat, CurrencyId, Balance>;

  #[pallet::storage]
  #[pallet::getter(fn account_borrows)]
  pub type AccountStakes<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    Blake2_128Concat,
    CurrencyId,
    Stake<Balance>,
    ValueQuery,
  >;

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// The assets get staked successfully
    Staked(T::AccountId, CurrencyId, Balance),
    /// The derivative get unstaked successfully
    Unstaked(T::AccountId, CurrencyId, Balance, Balance),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {}

  // Dispatchable functions allows users to interact with the pallet and invoke state changes.
  // These functions materialize as "extrinsics", which are often compared to transactions.
  // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// AccountID request withdrawal.
    /// This will dispatch an Event on the chain and the Quprum should listen to process the job
    /// and send the confirmation once done.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::stake())]
    pub fn stake(
      origin: OriginFor<T>,
      currency_id: CurrencyId,
      amount: Balance,
      duration: u32,
    ) -> DispatchResultWithPostInfo {
      let account_id = ensure_signed(origin)?;

      // FIXME: Maybe we should have some way to open / close a market of staking

      // transfer the assets to the pallet account id
      T::CurrencyWrapr::transfer(currency_id, &account_id, &Self::account_id(), amount, true)?;
      AccountStakes::<T>::insert(
        account_id.clone(),
        currency_id,
        Stake {
          initial_balance: amount,
          principal: amount,
          duration,
        },
      );
      // Update our staking pool
      StakingPool::<T>::try_mutate(currency_id, |balance| -> DispatchResult {
        if let Some(b) = balance {
          *balance = Some(b.checked_add(amount).ok_or(ArithmeticError::Overflow)?);
        } else {
          *balance = Some(1)
        }
        Ok(())
      })?;

      Self::deposit_event(Event::<T>::Staked(account_id, currency_id, amount));
      Ok(().into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
      <T as pallet::Config>::PalletId::get().into_account()
    }

    // Get all stakes for the account, serialized for quick RPC call
    pub fn get_account_stakes(account_id: &T::AccountId) -> Vec<(CurrencyId, Stake<BalanceInfo>)> {
      AccountStakes::<T>::iter_prefix(account_id)
        .map(|(currency_id, stake)| {
          (
            currency_id,
            Stake::<BalanceInfo> {
              principal: BalanceInfo {
                amount: stake.principal,
              },
              initial_balance: BalanceInfo {
                amount: stake.initial_balance,
              },
              duration: stake.duration,
            },
          )
        })
        .collect()
    }
  }
}
