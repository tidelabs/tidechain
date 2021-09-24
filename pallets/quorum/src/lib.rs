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
  use frame_system::{pallet_prelude::*, RawOrigin};
  use sp_runtime::traits::{AccountIdConversion, StaticLookup};
  use tidefi_primitives::{
    pallet::QuorumExt, AssetId, Balance, CurrencyId, RequestId, Trade, TradeStatus, Withdrawal,
    WithdrawalStatus,
  };

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config:
    frame_system::Config + pallet_assets::Config<AssetId = AssetId, Balance = Balance>
  {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    #[pallet::constant]
    type QuorumPalletId: Get<PalletId>;
    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;

    type CurrencyWrapr: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  /// Quorum is enabled
  #[pallet::storage]
  #[pallet::getter(fn is_quorum_enabled)]
  pub(super) type QuorumStatus<T: Config> = StorageValue<_, bool, ValueQuery>;

  /// Quorum Account ID
  #[pallet::storage]
  #[pallet::getter(fn quorum_account_id)]
  pub type QuorumAccountId<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

  /// Requests counter
  #[pallet::storage]
  pub type RequestCounter<T: Config> = StorageValue<_, RequestId, ValueQuery>;

  /// Mapping of pending Withdrawals
  #[pallet::storage]
  #[pallet::getter(fn withdrawals)]
  pub type Withdrawals<T: Config> =
    StorageMap<_, Blake2_128Concat, RequestId, Withdrawal<T::AccountId, T::BlockNumber>>;

  /// Mapping of pending Trades
  #[pallet::storage]
  #[pallet::getter(fn trades)]
  pub type Trades<T: Config> =
    StorageMap<_, Blake2_128Concat, RequestId, Trade<T::AccountId, T::BlockNumber>>;

  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    pub quorum_enabled: bool,
    pub quorum_account: T::AccountId,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        quorum_enabled: true,
        quorum_account: T::QuorumPalletId::get().into_account(),
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      QuorumStatus::<T>::put(self.quorum_enabled);
      QuorumAccountId::<T>::put(self.quorum_account.clone());
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Quorum status changed
    /// [is_enabled]
    StatusChanged(bool),
    /// Quorum account changed
    /// [account_id]
    AccountChanged(T::AccountId),
    /// Quorum minted token to the account
    /// [sender, asset_id, amount]
    Minted(T::AccountId, CurrencyId, Balance),
    /// Quorum burned token to the account
    /// [sender, asset_id, amount]
    Burned(T::AccountId, CurrencyId, Balance),
    /// Quorum traded token to the account
    /// [sender, account_id, token_from, token_amount_from, token_to, token_amount_to]
    Traded(
      RequestId,
      T::AccountId,
      CurrencyId,
      Balance,
      CurrencyId,
      Balance,
    ),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// The Quorum is paused. Try again later.
    QuorumPaused,
    /// The access to the Quorum pallet is not allowed for this account ID.
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
    /// Quorum have confirmation and make a new deposit for the asset.
    ///
    /// If the asset id do not exist, it get created.
    ///
    /// - `account_id`: Account Id to be deposited.
    /// - `asset_id`: the asset to be deposited.
    /// - `mint_amount`: the amount to be deposited.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::mint())]
    pub fn mint(
      origin: OriginFor<T>,
      account_id: T::AccountId,
      asset_id: CurrencyId,
      mint_amount: Balance,
    ) -> DispatchResultWithPostInfo {
      // make sure it's the quorum
      let sender = ensure_signed(origin)?;
      ensure!(
        sender == Self::quorum_account_id(),
        Error::<T>::AccessDenied
      );

      // make sure the currency exists, the pallet failed if already exist but we don't really care.
      // FIXME: Maybe we could check if the failed is because of the asset already exist.
      // otherwise we should failed here
      if let CurrencyId::Wrapped(asset) = asset_id {
        let _force_create = pallet_assets::Pallet::<T>::force_create(
          RawOrigin::Root.into(),
          asset,
          // make the pallet account id the owner, so only this pallet can handle the funds.
          T::Lookup::unlookup(Self::account_id()),
          true,
          1,
        );
      }

      // mint the token
      T::CurrencyWrapr::mint_into(asset_id, &account_id, mint_amount)?;

      // send event to the chain
      Self::deposit_event(Event::<T>::Minted(account_id, asset_id, mint_amount));

      Ok(().into())
    }

    /// Quorum have confirmation and make a new burn (widthdraw).
    ///
    /// - `request_id`: Request ID.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::confirm_withdrawal())]
    pub fn confirm_withdrawal(
      origin: OriginFor<T>,
      request_id: RequestId,
    ) -> DispatchResultWithPostInfo {
      // make sure the quorum is not paused
      Self::ensure_not_paused()?;
      // make sure it's the quorum
      let sender = ensure_signed(origin)?;
      ensure!(
        sender == Self::quorum_account_id(),
        Error::<T>::AccessDenied
      );

      Withdrawals::<T>::try_mutate_exists(request_id, |withdrawal| {
        match withdrawal {
          None => {
            return Err(Error::<T>::InvalidRequestId);
          }
          Some(withdrawal) => {
            // remove the token from the account
            T::CurrencyWrapr::burn_from(
              withdrawal.asset_id,
              &withdrawal.account_id,
              withdrawal.amount,
            )
            .map_err(|_| Error::<T>::BurnFailed)?;

            Self::deposit_event(Event::<T>::Burned(
              withdrawal.account_id.clone(),
              withdrawal.asset_id,
              withdrawal.amount,
            ));
          }
        }
        // it deletes the item if mutated to a None.
        *withdrawal = None;
        Ok(())
      })?;

      Ok(().into())
    }

    /// Quorum have confirmation and make a new burn (widthdraw).
    ///
    /// - `request_id`: Request ID.
    /// - `amounts_from`: Amounts from the market markers.
    /// - `accounts_to`: Accounts of the market markers.
    /// - `amounts_to`: Request ID.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::confirm_withdrawal())]
    pub fn confirm_trade(
      origin: OriginFor<T>,
      request_id: RequestId,
      amounts_from: Vec<Balance>,
      accounts_to: Vec<T::AccountId>,
      amounts_to: Vec<Balance>,
    ) -> DispatchResultWithPostInfo {
      // make sure the quorum is not paused
      Self::ensure_not_paused()?;
      // make sure it's the quorum
      let sender = ensure_signed(origin)?;
      ensure!(
        sender == Self::quorum_account_id(),
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

    // FIXME: [@lemarier] Should be removed after the demo.
    //
    /// Quick trade for demo.
    ///
    /// - `account_id`: Account ID.
    /// - `asset_id_from`: Asset Id to send.
    /// - `amount_from`: Amount to send.
    /// - `asset_id_to`: Asset Id to receive.
    /// - `amount_to`: Amount to receive.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::confirm_withdrawal())]
    pub fn quick_trade(
      origin: OriginFor<T>,
      account_id: T::AccountId,
      asset_id_from: CurrencyId,
      amount_from: Balance,
      asset_id_to: CurrencyId,
      amount_to: Balance,
    ) -> DispatchResultWithPostInfo {
      // make sure the quorum is not paused
      Self::ensure_not_paused()?;
      // make sure it's the quorum
      let sender = ensure_signed(origin)?;
      ensure!(
        sender == Self::quorum_account_id(),
        Error::<T>::AccessDenied
      );

      // make sure the FROM balance is available
      match T::CurrencyWrapr::can_withdraw(asset_id_from, &account_id, amount_from) {
        WithdrawConsequence::Success => {
          // make sure we can deposit before burning
          T::CurrencyWrapr::can_deposit(asset_id_to, &account_id, amount_to)
            .into_result()
            .map_err(|_| Error::<T>::MintFailed)?;

          // burn from token
          T::CurrencyWrapr::burn_from(asset_id_from, &account_id, amount_from)
            .map_err(|_| Error::<T>::BurnFailed)?;

          // mint new tokens with fallback to restore token if it fails
          if T::CurrencyWrapr::mint_into(asset_id_to, &account_id, amount_to).is_err() {
            let revert = T::CurrencyWrapr::mint_into(asset_id_from, &account_id, amount_from);
            debug_assert!(revert.is_ok(), "withdrew funds previously; qed");
            return Err(Error::<T>::MintFailed.into());
          };

          // FIXME: we can probably remove this and only use the
          // event emitted by the Assets pallet
          // emit the burned event
          Self::deposit_event(Event::<T>::Traded(
            // fake request id
            0,
            account_id.clone(),
            asset_id_from,
            amount_from,
            asset_id_to,
            amount_to,
          ));
        }
        WithdrawConsequence::NoFunds => return Err(Error::<T>::NoFunds.into()),
        WithdrawConsequence::UnknownAsset => return Err(Error::<T>::UnknownAsset.into()),
        _ => return Err(Error::<T>::UnknownError.into()),
      };

      Ok(().into())
    }

    /// Quorum change status.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::set_status())]
    pub fn set_status(origin: OriginFor<T>, quorum_enabled: bool) -> DispatchResultWithPostInfo {
      // make sure it's the quorum
      let sender = ensure_signed(origin)?;
      ensure!(
        sender == Self::quorum_account_id(),
        Error::<T>::AccessDenied
      );

      // update quorum
      QuorumStatus::<T>::put(quorum_enabled);

      // emit event
      Self::deposit_event(Event::<T>::StatusChanged(quorum_enabled));

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
        sender == Self::quorum_account_id(),
        Error::<T>::AccessDenied
      );

      // update quorum
      QuorumAccountId::<T>::put(new_quorum.clone());

      // emit event
      Self::deposit_event(Event::<T>::AccountChanged(new_quorum));

      Ok(().into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
      T::QuorumPalletId::get().into_account()
    }

    /// Increments the cached request id and returns the value to be used.
    fn next_request_seed() -> RequestId {
      <RequestCounter<T>>::mutate(|counter| {
        *counter += 1;
        *counter
      })
    }

    fn ensure_not_paused() -> Result<(), DispatchError> {
      if Self::is_quorum_enabled() {
        Ok(())
      } else {
        Err(Error::<T>::QuorumPaused.into())
      }
    }
  }

  impl<T: Config> QuorumExt<T::AccountId, T::BlockNumber> for Pallet<T> {
    /// Get quorum status
    fn is_quorum_enabled() -> bool {
      Self::is_quorum_enabled()
    }

    /// Update quprum status
    fn set_quorum_status(quorum_enabled: bool) {
      // update quorum
      QuorumStatus::<T>::put(quorum_enabled);
      // emit event
      Self::deposit_event(Event::<T>::QuorumStatusChanged(quorum_enabled));
    }

    /// Add new withdrawal in queue
    fn add_new_withdrawal_in_queue(
      account_id: T::AccountId,
      asset_id: CurrencyId,
      amount: Balance,
      external_address: Vec<u8>,
    ) -> (RequestId, Withdrawal<T::AccountId, T::BlockNumber>) {
      let request_id = Self::next_request_seed();
      let withdrawal = Withdrawal {
        account_id,
        amount,
        asset_id,
        external_address,
        status: WithdrawalStatus::Pending,
        block_number: <frame_system::Pallet<T>>::block_number(),
      };

      // insert in our queue
      Withdrawals::<T>::insert(request_id, withdrawal.clone());

      // return values
      (request_id, withdrawal)
    }

    fn add_new_trade_in_queue(
      account_id: T::AccountId,
      asset_id_from: CurrencyId,
      amount_from: Balance,
      asset_id_to: CurrencyId,
      amount_to: Balance,
    ) -> (RequestId, Trade<T::AccountId, T::BlockNumber>) {
      let request_id = Self::next_request_seed();
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
