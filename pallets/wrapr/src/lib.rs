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
  use tidefi_primitives::{
    pallet::{AssetRegistryExt, OracleExt, QuorumExt},
    Balance, BalanceInfo, CurrencyId, Hash,
  };

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    #[pallet::constant]
    type PalletId: Get<PalletId>;

    /// Quorum traits
    type Quorum: QuorumExt<Self::AccountId, Self::BlockNumber>;

    /// Oracle traits
    type Oracle: OracleExt<Self::AccountId, Self::BlockNumber>;

    /// Asset registry traits
    type AssetRegistry: AssetRegistryExt;

    /// Our currencies manager
    type CurrencyWrapr: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

    /// Weight information for extrinsics in this pallet
    type WeightInfo: WeightInfo;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Event emitted when widthdraw is requested.
    /// [from_account_id, to_account_id, currency_id, amount]
    Transfer(T::AccountId, T::AccountId, CurrencyId, Balance),
    /// Event emitted when widthdraw is requested.
    /// [request_id, account, asset_id, amount, external_address]
    Withdrawal(Hash, T::AccountId, CurrencyId, Balance, Vec<u8>),
    /// Event emitted when trade is requested.
    /// [request_id, account, asset_id_from, amount_from, asset_id_to, amount_to]
    Trade(Hash, T::AccountId, CurrencyId, Balance, CurrencyId, Balance),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Asset is currently disabled
    DisabledAsset,
    /// Unknown Asset
    UnknownAsset,
    /// No Funds available for this Asset Id
    NoFunds,
    /// Unknown Error
    UnknownError,
    /// Quorum is paused. Withdrawal is not allowed
    QuorumPaused,
    /// Oracle is paused. Trading is not allowed
    OraclePaused,
  }

  // Dispatchable functions allows users to interact with the pallet and invoke state changes.
  // These functions materialize as "extrinsics", which are often compared to transactions.
  // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Transfer funds from one account into another.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::transfer())]
    pub fn transfer(
      origin: OriginFor<T>,
      destination_id: T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the transaction is signed
      let account_id = ensure_signed(origin)?;

      // 2. Make sure the currency is not disabled
      ensure!(
        T::AssetRegistry::is_enabled(currency_id),
        Error::<T>::DisabledAsset
      );

      // 3. Transfer the request currency, only if the funds are available and the recipient can receive it.
      T::CurrencyWrapr::transfer(currency_id, &account_id, &destination_id, amount, true)?;

      // 4. Send event to the chain
      Self::deposit_event(Event::<T>::Transfer(
        account_id,
        destination_id,
        currency_id,
        amount,
      ));
      Ok(().into())
    }

    /// AccountID request withdrawal of currency.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::request_withdrawal())]
    pub fn request_withdrawal(
      origin: OriginFor<T>,
      currency_id: CurrencyId,
      amount: Balance,
      external_address: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the transaction is signed
      let account_id = ensure_signed(origin)?;

      // 2. Make sure the quorum is enabled
      ensure!(T::Quorum::is_quorum_enabled(), Error::<T>::QuorumPaused);

      // 3. Make sure the currency is not disabled
      ensure!(
        T::AssetRegistry::is_enabled(currency_id),
        Error::<T>::DisabledAsset
      );

      // 4. Make sure the account have enough funds
      match T::CurrencyWrapr::can_withdraw(currency_id, &account_id, amount) {
        WithdrawConsequence::Success => {
          // Add withdrawal in queue
          let (withdrawal_id, _) = T::Quorum::add_new_withdrawal_in_queue(
            account_id.clone(),
            currency_id,
            amount,
            external_address.clone(),
          );
          // Send event to the chain
          Self::deposit_event(Event::<T>::Withdrawal(
            withdrawal_id,
            account_id,
            currency_id,
            amount,
            external_address,
          ));

          Ok(().into())
        }
        WithdrawConsequence::NoFunds => Err(Error::<T>::NoFunds.into()),
        WithdrawConsequence::UnknownAsset => Err(Error::<T>::UnknownAsset.into()),
        _ => Err(Error::<T>::UnknownError.into()),
      }
    }

    /// AccountID request trade.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::request_trade())]
    pub fn request_trade(
      origin: OriginFor<T>,
      currency_id_from: CurrencyId,
      amount_from: Balance,
      currency_id_to: CurrencyId,
      amount_to: Balance,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the transaction is signed
      let account_id = ensure_signed(origin)?;

      // 2. Make sure the oracle is enabled
      ensure!(T::Oracle::is_oracle_enabled(), Error::<T>::QuorumPaused);

      // 3. Make sure the `currency_id_from` is not disabled
      ensure!(
        T::AssetRegistry::is_enabled(currency_id_from),
        Error::<T>::DisabledAsset
      );

      // 4. Make sure the `currency_id_to` is not disabled
      ensure!(
        T::AssetRegistry::is_enabled(currency_id_to),
        Error::<T>::DisabledAsset
      );

      // 5. Make sure the account have enough funds for the `asset_id_from`
      match T::CurrencyWrapr::can_withdraw(currency_id_from, &account_id, amount_from) {
        WithdrawConsequence::Success => {
          // Add trade in queue
          let (trade_id, _) = T::Oracle::add_new_trade_in_queue(
            account_id.clone(),
            currency_id_from,
            amount_from,
            currency_id_to,
            amount_to,
          );
          // Send event to the chain
          Self::deposit_event(Event::<T>::Trade(
            trade_id,
            account_id,
            currency_id_from,
            amount_from,
            currency_id_to,
            amount_to,
          ));

          Ok(().into())
        }
        WithdrawConsequence::NoFunds => Err(Error::<T>::NoFunds.into()),
        WithdrawConsequence::UnknownAsset => Err(Error::<T>::UnknownAsset.into()),
        _ => Err(Error::<T>::UnknownError.into()),
      }
    }
  }

  impl<T: Config> Pallet<T> {
    /// Get account balance via custom RPC api using the currency wrapper.
    pub fn get_account_balance(
      asset_id: CurrencyId,
      account_id: &T::AccountId,
    ) -> Result<BalanceInfo, DispatchError> {
      let balance = T::CurrencyWrapr::balance(asset_id, account_id);
      Ok(BalanceInfo { amount: balance })
    }
  }
}
