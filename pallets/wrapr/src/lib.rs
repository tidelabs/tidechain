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
  };
  use frame_system::pallet_prelude::*;
  use sp_io::hashing::blake2_256;
  use tidefi_primitives::{
    pallet::{AssetRegistryExt, OracleExt, QuorumExt},
    Balance, CurrencyId, Hash,
  };

  /// Wrapr configuration
  #[pallet::config]
  pub trait Config: frame_system::Config {
    /// Events
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    /// Weights
    type WeightInfo: WeightInfo;

    /// Quorum traits
    type Quorum: QuorumExt<Self::AccountId, Self::BlockNumber>;

    /// Oracle traits
    type Oracle: OracleExt<Self::AccountId, Self::BlockNumber>;

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

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Event emitted when widthdraw is requested.
    /// \[from_account_id, to_account_id, currency_id, amount\]
    Transfer {
      from_account_id: T::AccountId,
      to_account_id: T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
    },
    /// Event emitted when widthdraw is requested.
    Withdrawal {
      request_id: Hash,
      account: T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
      external_address: Vec<u8>,
    },
    /// Event emitted when trade is requested.
    /// \[request_id, account, currency_id_from, amount_from, currency_id_to, amount_to\]
    Trade {
      request_id: Hash,
      account: T::AccountId,
      currency_id_from: CurrencyId,
      amount_from: Balance,
      currency_id_to: CurrencyId,
      amount_to: Balance,
      extrinsic_hash: [u8; 32],
    },
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Asset is currently disabled or do not exist on chain
    AssetDisabled,
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

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Transfer funds from one account into another.
    ///
    /// - `destination_id`: Destination account
    /// - `currency_id`: The currency to transfer
    /// - `amount`: The amount to transfer
    ///
    /// Emits `Transfer` event when successful.
    ///
    /// Weight: `O(1)`
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
        T::AssetRegistry::is_currency_enabled(currency_id),
        Error::<T>::AssetDisabled
      );

      // 3. Transfer the request currency, only if the funds are available and the recipient can receive it.
      T::CurrencyWrapr::transfer(currency_id, &account_id, &destination_id, amount, true)?;

      // 4. Send event to the chain
      Self::deposit_event(Event::<T>::Transfer {
        from_account_id: account_id,
        to_account_id: destination_id,
        currency_id,
        amount,
      });
      Ok(().into())
    }

    /// Request wrapped asset withdrawal.
    ///
    /// - `currency_id`: The currency to withdraw.
    /// - `amount`: The amount to transfer
    /// - `external_address`: External address where to send funds.
    ///
    /// Emits `Withdrawal` event when successful.
    ///
    /// Weight: `O(1)`
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
        T::AssetRegistry::is_currency_enabled(currency_id),
        Error::<T>::AssetDisabled
      );

      // 4. Make sure the currency not a TIDE as it's not supported.
      ensure!(currency_id != CurrencyId::Tide, Error::<T>::UnknownAsset);

      // 5. Make sure the account have enough funds
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
          Self::deposit_event(Event::<T>::Withdrawal {
            request_id: withdrawal_id,
            account: account_id,
            currency_id,
            amount,
            external_address,
          });

          Ok(().into())
        }
        WithdrawConsequence::NoFunds => Err(Error::<T>::NoFunds.into()),
        WithdrawConsequence::UnknownAsset => Err(Error::<T>::UnknownAsset.into()),
        _ => Err(Error::<T>::UnknownError.into()),
      }
    }

    /// Request trade (swap) trough the market makers.
    ///
    /// This will register a new request and will be queued for the oracle, do
    /// not expect an immediate response.
    ///
    /// - `currency_id_from`: The currency to send.
    /// - `amount_from`: The amount to send.
    /// - `currency_id_to`: The currency to receive.
    /// - `amount_to`: The expected amount to receive with a 10% margin.
    ///
    /// Emits `Trade` event when successful.
    ///
    /// Weight: `O(1)`
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
        T::AssetRegistry::is_currency_enabled(currency_id_from),
        Error::<T>::AssetDisabled
      );

      // 4. Make sure the `currency_id_to` is not disabled
      ensure!(
        T::AssetRegistry::is_currency_enabled(currency_id_to),
        Error::<T>::AssetDisabled
      );

      // 5. Grab the extrinsic hash of the current extrinsic for better traceability
      let extrinsic_hash = blake2_256(&<frame_system::Pallet<T>>::extrinsic_data(
        <frame_system::Pallet<T>>::extrinsic_index().ok_or(Error::<T>::UnknownError)?,
      ));

      // 6. Make sure the account have enough funds for the `asset_id_from`
      match T::CurrencyWrapr::can_withdraw(currency_id_from, &account_id, amount_from) {
        WithdrawConsequence::Success => {
          // 6. a) Add trade in queue
          let (trade_id, _) = T::Oracle::add_new_trade_in_queue(
            account_id.clone(),
            currency_id_from,
            amount_from,
            currency_id_to,
            amount_to,
            <frame_system::Pallet<T>>::block_number(),
            extrinsic_hash
          );

          // 6 b) Send event to the chain
          Self::deposit_event(Event::<T>::Trade {
            request_id: trade_id,
            account: account_id,
            currency_id_from,
            amount_from,
            currency_id_to,
            amount_to,
            extrinsic_hash,
          });

          Ok(().into())
        }
        WithdrawConsequence::NoFunds => Err(Error::<T>::NoFunds.into()),
        WithdrawConsequence::UnknownAsset => Err(Error::<T>::UnknownAsset.into()),
        _ => Err(Error::<T>::UnknownError.into()),
      }
    }
  }
}
