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
    PalletId,
  };
  use frame_system::pallet_prelude::*;
  use sp_runtime::traits::AccountIdConversion;
  use tidefi_primitives::{BalanceInfo};

  pub type AssetIdOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
  pub type BalanceOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config: frame_system::Config + pallet_quorum::Config + pallet_balances::Config {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    type Assets: Transfer<Self::AccountId> + Inspect<Self::AccountId> + Mutate<Self::AccountId>;
    #[pallet::constant]
    type PalletId: Get<PalletId>;
    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Event emitted when widthdraw is requested. The Quorum will process the wrapped
    /// refund (example BTC) and will send confirmation once completed.
    /// [sender, asset_id, amount]
    Withdrawal(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Unknown Asset.
    UnknownAsset,
    /// No Funds available for this Asset Id.
    NoFunds,
    /// Unknown Error.
    UnknownError,
    /// Quorum is paused. No new transaction are queued.
    QuorumPaused,
  }

  // Dispatchable functions allows users to interact with the pallet and invoke state changes.
  // These functions materialize as "extrinsics", which are often compared to transactions.
  // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// AccountID request withdrawal.
    /// This will dispatch an Event on the chain and the Quprum should listen to process the job
    /// and send the confirmation once done.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::request_withdrawal())]
    pub fn request_withdrawal(
      origin: OriginFor<T>,
      asset_id: AssetIdOf<T>,
      amount: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
      let who = ensure_signed(origin)?;
      // make sure the quorum is enabled
      ensure!(Self::is_quorum_enabled(), Error::<T>::QuorumPaused);
      // make sure the account have the fund to save some time
      // to the quorum
      match T::Assets::can_withdraw(asset_id, &who, amount) {
        WithdrawConsequence::Success => {
          // send event to the chain
          // FIXME: save it to a local cache and quorum can poll via RPC (much safier than listening to events)
          Self::deposit_event(Event::<T>::Withdrawal(who, asset_id, amount));

          //Self::add_to_withdrawals_queue(asset_id, &who, amount);

          // ok
          Ok(Pays::No.into())
        }
        WithdrawConsequence::NoFunds => Err(Error::<T>::NoFunds.into()),
        WithdrawConsequence::UnknownAsset => Err(Error::<T>::UnknownAsset.into()),
        _ => Err(Error::<T>::UnknownError.into()),
      }
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config + pallet_quorum::Config + pallet_balances::Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
      <T as pallet::Config>::PalletId::get().into_account()
    }

    // rpc, it need to be serializable
    pub fn get_account_balance(
      asset_id: T::AssetId,
      account_id: &T::AccountId,
    ) -> Result<BalanceInfo, DispatchError>
    where
      u128: From<<T as pallet_balances::Config>::Balance>,
    {
      let balance: u128 = if asset_id == 0 {
        pallet_balances::Pallet::<T>::free_balance(account_id).into()
      } else {
        pallet_assets::Pallet::<T>::balance(asset_id, account_id)
      };

      Ok(BalanceInfo { amount: balance })
    }

    pub fn is_quorum_enabled() -> bool {
      pallet_quorum::Pallet::<T>::is_quorum_enabled()
    }
  }
}
