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
    traits::{
      fungibles::{Inspect, Mutate, Transfer},
      tokens::WithdrawConsequence,
    },
    PalletId,
  };
  use frame_system::pallet_prelude::*;
  use sp_runtime::traits::AccountIdConversion;
  use tidefi_primitives::{
    pallet::{OracleExt, SecurityExt},
    AssetId, Balance, CurrencyId, Hash, Trade, TradeStatus,
  };

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config:
    frame_system::Config + pallet_assets::Config<AssetId = AssetId, Balance = Balance>
  {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    #[pallet::constant]
    type OraclePalletId: Get<PalletId>;
    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;

    /// Security traits
    type Security: SecurityExt<Self::AccountId, Self::BlockNumber>;

    type CurrencyWrapr: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  /// Oracle is enabled
  #[pallet::storage]
  #[pallet::getter(fn is_oracle_enabled)]
  pub(super) type OracleStatus<T: Config> = StorageValue<_, bool, ValueQuery>;

  /// Oracle Account ID
  #[pallet::storage]
  #[pallet::getter(fn oracle_account_id)]
  pub type OracleAccountId<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

  /// Mapping of pending Trades
  #[pallet::storage]
  #[pallet::getter(fn trades)]
  pub type Trades<T: Config> =
    StorageMap<_, Blake2_128Concat, Hash, Trade<T::AccountId, T::BlockNumber>>;

  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    pub enabled: bool,
    pub account: T::AccountId,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        enabled: true,
        account: T::OraclePalletId::get().into_account(),
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      OracleStatus::<T>::put(self.enabled);
      OracleAccountId::<T>::put(self.account.clone());
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Oracle status changed
    /// [is_enabled]
    StatusChanged(bool),
    /// Oracle account changed
    /// [account_id]
    AccountChanged(T::AccountId),
    /// Oracle confirmed trade
    /// [request_id, account_id, token_from, token_amount_from, token_to, token_amount_to]
    Traded(Hash, T::AccountId, CurrencyId, Balance, CurrencyId, Balance),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// The Quorum is paused. Try again later.
    OraclePaused,
    /// The access to the Oracle pallet is not allowed for this account ID.
    AccessDenied,
    /// Invalid request ID.
    InvalidRequestId,
    /// Invalid request status.
    InvalidRequestStatus,
    /// There is a conflict in the request.
    Conflict,
    /// Unable to burn token.
    BurnFailed,
    /// Unable to mint token.
    MintFailed,
    /// Unknown Asset.
    UnknownAsset,
    /// No Funds available for this Asset Id.
    NoFunds,
    /// Unknown Error.
    UnknownError,
  }

  // Dispatchable functions allows users to interact with the pallet and invoke state changes.
  // These functions materialize as "extrinsics", which are often compared to transactions.
  // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Oracle have confirmation and make a new burn (widthdraw).
    ///
    /// - `request_id`: Request ID.
    /// - `amounts_from`: Amounts from the market markers.
    /// - `accounts_to`: Accounts of the market markers.
    /// - `amounts_to`: Request ID.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::confirm_withdrawal())]
    pub fn confirm_trade(
      origin: OriginFor<T>,
      request_id: Hash,
      amounts_from: Vec<Balance>,
      accounts_to: Vec<T::AccountId>,
      amounts_to: Vec<Balance>,
    ) -> DispatchResultWithPostInfo {
      // make sure the oracle is not paused
      Self::ensure_not_paused()?;

      // make sure it's the oracle account
      let sender = ensure_signed(origin)?;
      ensure!(
        sender == Self::oracle_account_id(),
        Error::<T>::AccessDenied
      );

      Trades::<T>::try_mutate_exists(request_id, |trade| {
        match trade {
          None => {
            return Err(Error::<T>::InvalidRequestId);
          }
          Some(trade) => {
            if trade.status != TradeStatus::Pending {
              return Err(Error::<T>::InvalidRequestStatus);
            }

            // check total_from
            let mut total_from: Balance = 0;
            for amt_from in amounts_from.iter() {
              total_from += amt_from;
            }
            if trade.amount_from != total_from {
              return Err(Error::<T>::Conflict);
            }

            // check amounts_to
            // allowed to go +/- 10%
            let mut total_to: Balance = 0;
            for amt_to in amounts_to.iter() {
              total_to += amt_to;
            }
            if total_to * 10 < trade.amount_to * 9 || total_to * 10 > trade.amount_to * 11 {
              return Err(Error::Conflict);
            }

            // make sure the FROM balance is available
            match T::CurrencyWrapr::can_withdraw(
              trade.token_from,
              &trade.account_id,
              trade.amount_from,
            ) {
              WithdrawConsequence::Success => {
                let mut total_to = 0;
                // make sure all the market markers have enough funds
                for (pos, amt) in amounts_to.iter().enumerate() {
                  total_to += amt;
                  match T::CurrencyWrapr::can_withdraw(trade.token_to, &accounts_to[pos], *amt) {
                    // do nothing, we can continue
                    WithdrawConsequence::Success => continue,
                    // no funds error
                    WithdrawConsequence::NoFunds => return Err(Error::NoFunds),
                    // unknown assets
                    WithdrawConsequence::UnknownAsset => return Err(Error::UnknownAsset),
                    // throw an error, we really need a success here
                    _ => return Err(Error::UnknownError),
                  }
                }

                // make sure we can deposit before burning
                T::CurrencyWrapr::can_deposit(trade.token_to, &trade.account_id, total_to)
                  .into_result()
                  .map_err(|_| Error::<T>::MintFailed)?;

                // burn from token
                T::CurrencyWrapr::burn_from(trade.token_from, &trade.account_id, trade.amount_from)
                  .map_err(|_| Error::<T>::BurnFailed)?;

                // mint new tokens with fallback to restore token if it fails
                if T::CurrencyWrapr::mint_into(trade.token_to, &trade.account_id, total_to).is_err()
                {
                  let revert = T::CurrencyWrapr::mint_into(
                    trade.token_from,
                    &trade.account_id,
                    trade.amount_from,
                  );
                  debug_assert!(revert.is_ok(), "withdrew funds previously; qed");
                  return Err(Error::<T>::MintFailed);
                };
                // remove tokens from the MM accounts
                for (pos, amt) in amounts_to.iter().enumerate() {
                  // remove token_to from acc
                  T::CurrencyWrapr::burn_from(trade.token_to, &accounts_to[pos], *amt)
                    .map_err(|_| Error::<T>::BurnFailed)?;
                  // add token_from to acc
                  // using amounts_from
                  T::CurrencyWrapr::mint_into(
                    trade.token_from,
                    &accounts_to[pos],
                    amounts_from[pos],
                  )
                  .map_err(|_| Error::<T>::BurnFailed)?;
                }
                // FIXME: we can probably remove this and only use the
                // event emitted by the Assets pallet
                // emit the burned event
                Self::deposit_event(Event::<T>::Traded(
                  request_id,
                  trade.account_id.clone(),
                  trade.token_from,
                  trade.amount_from,
                  trade.token_to,
                  trade.amount_to,
                ));
              }
              WithdrawConsequence::NoFunds => return Err(Error::<T>::NoFunds),
              WithdrawConsequence::UnknownAsset => return Err(Error::<T>::UnknownAsset),
              _ => return Err(Error::<T>::UnknownError),
            };
          }
        }
        // it deletes the item if mutated to a None.
        *trade = None;
        Ok(())
      })?;

      Ok(().into())
    }

    /// Quorum change account ID.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::set_status())]
    pub fn set_account_id(
      origin: OriginFor<T>,
      new_quorum: T::AccountId,
    ) -> DispatchResultWithPostInfo {
      // make sure it's the quorum
      let sender = ensure_signed(origin)?;
      ensure!(
        sender == Self::oracle_account_id(),
        Error::<T>::AccessDenied
      );

      // update oracle
      OracleAccountId::<T>::put(new_quorum.clone());

      // emit event
      Self::deposit_event(Event::<T>::AccountChanged(new_quorum));

      Ok(().into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
      T::OraclePalletId::get().into_account()
    }

    fn ensure_not_paused() -> Result<(), DispatchError> {
      if Self::is_oracle_enabled() {
        Ok(())
      } else {
        Err(Error::<T>::OraclePaused.into())
      }
    }
  }

  impl<T: Config> OracleExt<T::AccountId, T::BlockNumber> for Pallet<T> {
    /// Get oracle status
    fn is_oracle_enabled() -> bool {
      T::Security::is_chain_running() && Self::is_oracle_enabled()
    }

    /// Get currency status
    fn is_currency_enabled(currency_id: CurrencyId) -> bool {
      match currency_id {
        CurrencyId::Tide => {
          // FIXME: Should we allow disabling of TIDE token?
          true
        }
        CurrencyId::Wrapped(asset) => {
          // make sure currency is created
          false
        }
      }
    }

    fn add_new_trade_in_queue(
      account_id: T::AccountId,
      asset_id_from: CurrencyId,
      amount_from: Balance,
      asset_id_to: CurrencyId,
      amount_to: Balance,
    ) -> (Hash, Trade<T::AccountId, T::BlockNumber>) {
      let request_id = T::Security::get_unique_id(account_id.clone());
      let trade = Trade {
        account_id,
        token_from: asset_id_from,
        token_to: asset_id_to,
        amount_from,
        amount_to,
        status: TradeStatus::Pending,
        block_number: <frame_system::Pallet<T>>::block_number(),
      };

      // insert in our queue
      Trades::<T>::insert(request_id, trade.clone());

      // return values
      (request_id, trade)
    }
  }
}
