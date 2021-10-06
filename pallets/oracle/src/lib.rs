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

  /// Oracle configuration
  #[pallet::config]
  pub trait Config:
    frame_system::Config + pallet_assets::Config<AssetId = AssetId, Balance = Balance>
  {
    /// Events
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    /// Pallet ID
    #[pallet::constant]
    type OraclePalletId: Get<PalletId>;

    /// Weights
    type WeightInfo: WeightInfo;

    /// Security traits
    type Security: SecurityExt<Self::AccountId, Self::BlockNumber>;

    /// Currency wrapr
    type CurrencyWrapr: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  /// Oracle is enabled
  #[pallet::storage]
  #[pallet::getter(fn status)]
  pub(super) type OracleStatus<T: Config> = StorageValue<_, bool, ValueQuery>;

  /// Oracle Account ID
  #[pallet::storage]
  #[pallet::getter(fn account_id)]
  pub type OracleAccountId<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

  /// Mapping of pending Trades
  #[pallet::storage]
  #[pallet::getter(fn trades)]
  pub type Trades<T: Config> =
    StorageMap<_, Blake2_128Concat, Hash, Trade<T::AccountId, T::BlockNumber>>;

  /// Genesis configuration
  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    /// Oracle status
    pub enabled: bool,
    /// Oracle Account ID. Multisig is supported.
    /// This account will be able to confirm trades on-chain.
    pub account: T::AccountId,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        // Oracle is enabled by default
        enabled: true,
        // We use pallet account ID by default,
        // but should always be set in the genesis config.
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
    /// Oracle status changed \[is_enabled\]
    StatusChanged(bool),
    /// Oracle account changed \[account_id\]
    AccountChanged(T::AccountId),
    /// Oracle confirmed trade
    /// \[request_id, account_id, token_from, token_amount_from, token_to, token_amount_to\]
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

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Oracle have confirmation and confirm the trade.
    ///
    /// - `request_id`: Unique request ID.
    /// - `amounts_from`: Amounts from the market markers.
    /// - `accounts_to`: Accounts of the market markers.
    /// - `amounts_to`: Amount of the final asset to allocate from the market makers.
    ///
    /// Emits `Traded` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::confirm_trade())]
    pub fn confirm_trade(
      origin: OriginFor<T>,
      request_id: Hash,
      amounts_from: Vec<Balance>,
      accounts_to: Vec<T::AccountId>,
      amounts_to: Vec<Balance>,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the oracle/chain is not paused
      Self::ensure_not_paused()?;

      // 2. Make sure this is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(sender == Self::account_id(), Error::<T>::AccessDenied);

      // 3. Make sure the `request_id` exist
      Trades::<T>::try_mutate_exists(request_id, |trade| {
        match trade {
          None => {
            return Err(Error::<T>::InvalidRequestId);
          }
          Some(trade) => {
            // 4. Make sure the trade status is pending
            if trade.status != TradeStatus::Pending {
              return Err(Error::<T>::InvalidRequestStatus);
            }

            // 5. Check `amounts_from`
            let mut total_from: Balance = 0;
            for amt_from in amounts_from.iter() {
              total_from += amt_from;
            }
            if trade.amount_from != total_from {
              return Err(Error::<T>::Conflict);
            }

            // 6. Check `amounts_to`
            let mut total_to: Balance = 0;
            for amt_to in amounts_to.iter() {
              total_to += amt_to;
            }
            if total_to * 10 < trade.amount_to * 9 || total_to * 10 > trade.amount_to * 11 {
              return Err(Error::Conflict);
            }

            // 7. Make sure the `account_id` can withdraw the funds
            match T::CurrencyWrapr::can_withdraw(
              trade.token_from,
              &trade.account_id,
              trade.amount_from,
            ) {
              WithdrawConsequence::Success => {
                let mut total_to = 0;

                // 8. Make sure all the market markers have enough funds before we can continue
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

                // 9. Make sure we can deposit before burning
                T::CurrencyWrapr::can_deposit(trade.token_to, &trade.account_id, total_to)
                  .into_result()
                  .map_err(|_| Error::<T>::MintFailed)?;

                // 10. Burn token
                T::CurrencyWrapr::burn_from(trade.token_from, &trade.account_id, trade.amount_from)
                  .map_err(|_| Error::<T>::BurnFailed)?;

                // 11. Mint new tokens with fallback to restore token if it fails
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

                // 12. Remove tokens from the market makers accounts only if previous step succeed
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

                // 13. Emit event on chain
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
        // FIXME: do we want to keep a copy and update the status to completed?
        *trade = None;
        Ok(())
      })?;

      Ok(().into())
    }

    /// Oracle change the account ID who can confirm trade.
    ///
    /// Make sure to have access to the `account_id` otherwise
    /// only `root` will be able to update the oracle account.
    ///
    /// - `new_account_id`: The new Oracle account id.
    ///
    /// Emits `AccountChanged` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::set_account_id())]
    pub fn set_account_id(
      origin: OriginFor<T>,
      new_account_id: T::AccountId,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure this is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(sender == Self::account_id(), Error::<T>::AccessDenied);

      // 2. Update oracle account
      OracleAccountId::<T>::put(new_account_id.clone());

      // 3. Emit event on chain
      Self::deposit_event(Event::<T>::AccountChanged(new_account_id));

      Ok(().into())
    }

    /// Change Oracle status.
    ///
    /// - `is_enabled`: Is the oracle enabled?
    ///
    /// Emits `StatusChanged` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::weight(<T as pallet::Config>::WeightInfo::set_status())]
    pub fn set_status(origin: OriginFor<T>, is_enabled: bool) -> DispatchResultWithPostInfo {
      // 1. Make sure this is signed by `account_id`
      let sender = ensure_signed(origin)?;
      ensure!(sender == Self::account_id(), Error::<T>::AccessDenied);

      // 2. Update oracle status
      OracleStatus::<T>::set(is_enabled);

      // 3. Emit event on chain
      Self::deposit_event(Event::<T>::StatusChanged(is_enabled));

      Ok(().into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config> Pallet<T> {
    fn ensure_not_paused() -> Result<(), DispatchError> {
      if Self::is_oracle_enabled() {
        Ok(())
      } else {
        Err(Error::<T>::OraclePaused.into())
      }
    }
  }

  // implement the `OracleExt` functions
  impl<T: Config> OracleExt<T::AccountId, T::BlockNumber> for Pallet<T> {
    fn is_oracle_enabled() -> bool {
      // make sure the chain and the oracle pallet are enabled
      T::Security::is_chain_running() && Self::status()
    }

    fn add_new_trade_in_queue(
      account_id: T::AccountId,
      asset_id_from: CurrencyId,
      amount_from: Balance,
      asset_id_to: CurrencyId,
      amount_to: Balance,
    ) -> (Hash, Trade<T::AccountId, T::BlockNumber>) {
      // unique request id
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
