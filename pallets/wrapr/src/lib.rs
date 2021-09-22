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
    traits::tokens::{
      fungibles::{Inspect, Mutate, Transfer},
      WithdrawConsequence,
    },
    PalletId,
  };
  use frame_system::pallet_prelude::*;
  use sp_runtime::traits::AccountIdConversion;
  use tidefi_primitives::{pallet::QuorumExt, BalanceInfo, RequestId};

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
    /// Quorum traits.
    type Quorum: QuorumExt<Self::AccountId, AssetIdOf<Self>, BalanceOf<Self>, Self::BlockNumber>;
    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Event emitted when widthdraw is requested.
    /// [request_id, account, asset_id, amount]
    Withdrawal(RequestId, T::AccountId, AssetIdOf<T>, BalanceOf<T>),
    /// Event emitted when stake is requested.
    /// [request_id, account, asset_id_from, amount_from, asset_id_to, amount_to]
    Trade(
      RequestId,
      T::AccountId,
      AssetIdOf<T>,
      BalanceOf<T>,
      AssetIdOf<T>,
      BalanceOf<T>,
    ),
    /// Event emitted when stake is requested.
    /// [request_id, account, asset_id, duration]
    Stake(RequestId, T::AccountId, AssetIdOf<T>, u32),
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
      external_address: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
      let account_id = ensure_signed(origin)?;
      // make sure the quorum is enabled
      ensure!(Self::is_quorum_enabled(), Error::<T>::QuorumPaused);
      // make sure the account have the fund to save some time
      // to the quorum
      match T::Assets::can_withdraw(asset_id, &account_id, amount) {
        WithdrawConsequence::Success => {
          // add to the queue
          let (withdrawal_id, _) = T::Quorum::add_new_withdrawal_in_queue(
            account_id.clone(),
            asset_id,
            amount,
            external_address,
          );
          // send event to the chain
          Self::deposit_event(Event::<T>::Withdrawal(
            withdrawal_id,
            account_id,
            asset_id,
            amount,
          ));
          // ok
          Ok(Pays::No.into())
        }
        WithdrawConsequence::NoFunds => Err(Error::<T>::NoFunds.into()),
        WithdrawConsequence::UnknownAsset => Err(Error::<T>::UnknownAsset.into()),
        _ => Err(Error::<T>::UnknownError.into()),
      }
    }

    /// AccountID request trade.
    /// This will dispatch an Event on the chain and the Quprum should listen to process the job
    /// and send the confirmation once done.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::request_withdrawal())]
    pub fn request_trade(
      origin: OriginFor<T>,
      asset_id_from: AssetIdOf<T>,
      amount_from: BalanceOf<T>,
      asset_id_to: AssetIdOf<T>,
      amount_to: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
      let account_id = ensure_signed(origin)?;
      // make sure the quorum is enabled
      ensure!(Self::is_quorum_enabled(), Error::<T>::QuorumPaused);
      // make sure the account have the fund to save some time
      // to the quorum
      match T::Assets::can_withdraw(asset_id_from, &account_id, amount_from) {
        WithdrawConsequence::Success => {
          // add to the queue
          let (trade_id, _) = T::Quorum::add_new_trade_in_queue(
            account_id.clone(),
            asset_id_from,
            amount_from,
            asset_id_to,
            amount_to,
          );
          // send event to the chain
          Self::deposit_event(Event::<T>::Trade(
            trade_id,
            account_id,
            asset_id_from,
            amount_from,
            asset_id_to,
            amount_to,
          ));
          // ok
          Ok(Pays::No.into())
        }
        WithdrawConsequence::NoFunds => Err(Error::<T>::NoFunds.into()),
        WithdrawConsequence::UnknownAsset => Err(Error::<T>::UnknownAsset.into()),
        _ => Err(Error::<T>::UnknownError.into()),
      }
    }

    /// AccountID request stake.
    /// This will dispatch an Event on the chain and the Quprum should listen to process the job
    /// and send the confirmation once done.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::request_withdrawal())]
    pub fn request_stake(
      origin: OriginFor<T>,
      asset_id: AssetIdOf<T>,
      amount: BalanceOf<T>,
      duration: u32,
    ) -> DispatchResultWithPostInfo {
      let account_id = ensure_signed(origin)?;
      // make sure the quorum is enabled
      ensure!(Self::is_quorum_enabled(), Error::<T>::QuorumPaused);
      // make sure the account have the fund to save some time
      // to the quorum
      match T::Assets::can_withdraw(asset_id, &account_id, amount) {
        WithdrawConsequence::Success => {
          // add to the queue
          let (stake_id, _) =
            T::Quorum::add_new_stake_in_queue(account_id.clone(), asset_id, amount, duration);
          // send event to the chain
          Self::deposit_event(Event::<T>::Stake(stake_id, account_id, asset_id, duration));
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
      T::Quorum::is_quorum_enabled()
    }
  }
}
