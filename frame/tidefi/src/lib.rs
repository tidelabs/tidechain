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
      fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
      WithdrawConsequence,
    },
  };
  use frame_system::pallet_prelude::*;
  use sp_io::hashing::blake2_256;
  use sp_runtime::{
    traits::{CheckedDiv, Saturating},
    Permill,
  };
  use tidefi_primitives::{
    pallet::{AssetRegistryExt, FeesExt, OracleExt, QuorumExt, SecurityExt, SunriseExt},
    Balance, CurrencyId, EraIndex, Hash, SwapType,
  };

  /// Tidefi configuration
  #[pallet::config]
  pub trait Config: frame_system::Config {
    /// Events
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    /// Weights
    type WeightInfo: WeightInfo;

    /// Quorum traits
    type Quorum: QuorumExt<Self::AccountId, Self::BlockNumber>;

    /// Oracle traits
    type Oracle: OracleExt<Self::AccountId, Self::BlockNumber>;

    /// Asset registry traits
    type AssetRegistry: AssetRegistryExt;

    /// Fees traits
    type Fees: FeesExt<Self::AccountId, Self::BlockNumber>;

    /// Tidefi sunrise traits
    type Sunrise: SunriseExt<Self::AccountId, Self::BlockNumber>;

    /// Security traits
    type Security: SecurityExt<Self::AccountId, Self::BlockNumber>;

    /// Tidechain currency wrapper
    type CurrencyTidefi: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + InspectHold<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + MutateHold<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Event emitted when transfer is processed.
    Transfer {
      from_account_id: T::AccountId,
      to_account_id: T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
    },
    /// Event emitted when widthdraw is initialized.
    Withdrawal {
      account: T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
      external_address: Vec<u8>,
    },
    /// Event emitted when swap is initialized.
    Swap {
      request_id: Hash,
      account: T::AccountId,
      currency_id_from: CurrencyId,
      amount_from: Balance,
      currency_id_to: CurrencyId,
      amount_to: Balance,
      extrinsic_hash: [u8; 32],
      slippage_tolerance: Permill,
      swap_type: SwapType,
      is_market_maker: bool,
    },
    /// User cancelled the initial swap and the funds has been released
    SwapCancelled { request_id: Hash },
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Asset is currently disabled or do not exist on chain
    AssetDisabled,
    /// Swap between the same currency id
    SameCurrencyId,
    /// Cannot withdraw TDFY
    CannotWithdrawTdfy,
    /// No Funds available for this Asset Id
    NoFunds,
    /// Withdraw amount is greater than account balance
    WithdrawAmountGreaterThanAccountBalance,
    /// Withdraw amount is greater than asset supply
    WithdrawAmountGreaterThanAssetSupply,
    /// Asset account is frozen
    AccountAssetFrozen,
    /// Balance will become zero after withdrawal
    ReducedToZero,
    /// Unknown extrinsic index
    UnknownExtrinsicIndex,
    /// Unknown Error
    UnknownError,
    /// Quorum is paused. Withdrawal is not allowed
    QuorumPaused,
    /// Oracle is paused. Trading is not allowed
    OraclePaused,
    /// No active era
    NoActiveEra,
    /// Era is not ready to be claimed yet, try again later
    EraNotReady,
    /// Invalid era
    InvalidEra,
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
    #[pallet::call_index(0)]
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
      T::CurrencyTidefi::transfer(currency_id, &account_id, &destination_id, amount, false)?;

      // 4. Send event to the chain
      Self::deposit_event(Event::<T>::Transfer {
        from_account_id: account_id,
        to_account_id: destination_id,
        currency_id,
        amount,
      });
      Ok(().into())
    }

    /// Submit asset withdrawal to the Quorum.
    ///
    /// - `currency_id`: The currency to withdraw.
    /// - `amount`: The amount to transfer
    /// - `external_address`: External address where to send funds.
    ///
    /// Emits `Withdrawal` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::call_index(1)]
    #[pallet::weight(<T as pallet::Config>::WeightInfo::withdrawal())]
    pub fn withdrawal(
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

      // 4. Make sure the account have enough funds
      match T::CurrencyTidefi::can_withdraw(currency_id, &account_id, amount) {
        WithdrawConsequence::Success | WithdrawConsequence::ReducedToZero(_) => {
          // Add withdrawal in queue
          T::Quorum::add_new_withdrawal_in_queue(
            account_id.clone(),
            currency_id,
            amount,
            external_address.clone(),
          )?;
          // Send event to the chain
          Self::deposit_event(Event::<T>::Withdrawal {
            account: account_id,
            currency_id,
            amount,
            external_address,
          });

          Ok(().into())
        }
        WithdrawConsequence::NoFunds => {
          Err(Error::<T>::WithdrawAmountGreaterThanAccountBalance.into())
        }
        WithdrawConsequence::Underflow => {
          Err(Error::<T>::WithdrawAmountGreaterThanAssetSupply.into())
        }
        WithdrawConsequence::Frozen => Err(Error::<T>::AccountAssetFrozen.into()),
        _ => Err(Error::<T>::UnknownError.into()),
      }
    }

    /// Swap through the market makers.
    ///
    /// This will register a new request and will be queued for the oracle, do
    /// not expect an immediate response.
    ///
    /// - `currency_id_from`: The currency to send.
    /// - `amount_from`: The amount to send.
    /// - `currency_id_to`: The currency to receive.
    /// - `amount_to`: The expected amount to receive with a 10% margin.
    ///
    /// Emits `Swap` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::call_index(2)]
    #[pallet::weight(<T as pallet::Config>::WeightInfo::swap())]
    pub fn swap(
      origin: OriginFor<T>,
      currency_id_from: CurrencyId,
      amount_from: Balance,
      currency_id_to: CurrencyId,
      amount_to: Balance,
      swap_type: SwapType,
      slippage_tolerance: Option<Permill>,
    ) -> DispatchResultWithPostInfo {
      // Make sure the transaction is signed
      let account_id = ensure_signed(origin)?;

      // Make sure the oracle is enabled
      ensure!(T::Oracle::is_oracle_enabled(), Error::<T>::OraclePaused);

      // Make sure the `currency_id_from` is not disabled
      ensure!(
        T::AssetRegistry::is_currency_enabled(currency_id_from),
        Error::<T>::AssetDisabled
      );

      // Make sure the `currency_id_to` is not disabled
      ensure!(
        T::AssetRegistry::is_currency_enabled(currency_id_to),
        Error::<T>::AssetDisabled
      );

      // Make sure the swap currencies are different
      ensure!(
        currency_id_from != currency_id_to,
        Error::<T>::SameCurrencyId
      );

      // Grab the extrinsic hash of the current extrinsic for better traceability
      let extrinsic_hash = blake2_256(&<frame_system::Pallet<T>>::extrinsic_data(
        <frame_system::Pallet<T>>::extrinsic_index().ok_or(Error::<T>::UnknownExtrinsicIndex)?,
      ));

      // Validate if the user is a market maker when the swap is requested to allocate the correct fees
      let is_market_maker = T::Oracle::is_market_maker(account_id.clone())?;

      // Make sure the account have enough funds for the `asset_id_from`
      match T::CurrencyTidefi::can_withdraw(currency_id_from, &account_id, amount_from) {
        WithdrawConsequence::Success => {
          let mut real_slippage_tolerance = slippage_tolerance.unwrap_or(Permill::zero());
          if real_slippage_tolerance.is_zero() && swap_type == SwapType::Limit {
            real_slippage_tolerance = Permill::from_parts(1);
          }

          // a) Add trade in queue
          let (trade_id, _) = T::Oracle::add_new_swap_in_queue(
            account_id.clone(),
            currency_id_from,
            amount_from,
            currency_id_to,
            amount_to,
            <frame_system::Pallet<T>>::block_number(),
            extrinsic_hash,
            is_market_maker,
            swap_type.clone(),
            real_slippage_tolerance,
          )?;

          // b) Send event to the chain
          Self::deposit_event(Event::<T>::Swap {
            request_id: trade_id,
            account: account_id,
            currency_id_from,
            amount_from,
            currency_id_to,
            amount_to,
            extrinsic_hash,
            swap_type,
            is_market_maker,
            slippage_tolerance: real_slippage_tolerance,
          });

          Ok(().into())
        }
        WithdrawConsequence::NoFunds => {
          Err(Error::<T>::WithdrawAmountGreaterThanAccountBalance.into())
        }
        WithdrawConsequence::Underflow => {
          Err(Error::<T>::WithdrawAmountGreaterThanAssetSupply.into())
        }
        WithdrawConsequence::Frozen => Err(Error::<T>::AccountAssetFrozen.into()),
        WithdrawConsequence::ReducedToZero(_) => Err(Error::<T>::ReducedToZero.into()),
        _ => Err(Error::<T>::UnknownError.into()),
      }
    }

    /// Cancel swap and release funds.
    ///
    /// This will cancel a swap request and release remaining funds, if the swap is partially filled.
    ///
    /// - `request_id`: The request ID to cancel.
    ///
    /// Emits `SwapCancelled` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::call_index(3)]
    #[pallet::weight(<T as pallet::Config>::WeightInfo::swap())]
    pub fn cancel_swap(origin: OriginFor<T>, request_id: Hash) -> DispatchResultWithPostInfo {
      // 1. Make sure the transaction is signed
      let account_id = ensure_signed(origin)?;

      // 2. Make sure the oracle is enabled
      ensure!(T::Oracle::is_oracle_enabled(), Error::<T>::OraclePaused);

      // 3. Remove the swap request and release funds if needed
      T::Oracle::remove_swap_from_queue(account_id, request_id)?;

      // 4. Emit event on chain
      Self::deposit_event(Event::<T>::SwapCancelled { request_id });

      Ok(().into())
    }

    /// Claim available sunrise rewards
    ///
    /// - `era_index`: Era to claim rewards
    ///
    /// Emits `RewardsClaimed` event when successful.
    ///
    /// Weight: `O(1)`
    #[pallet::call_index(4)]
    #[pallet::weight(<T as pallet::Config>::WeightInfo::claim_sunrise_rewards())]
    pub fn claim_sunrise_rewards(
      origin: OriginFor<T>,
      era_index: EraIndex,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the transaction is signed
      let account_id = ensure_signed(origin)?;

      // 2. Make sure the era Index provided is ready to be claimed
      let current_era = T::Fees::current_era().ok_or(Error::<T>::NoActiveEra)?;

      // Unable to claim current Era
      if era_index >= current_era.index {
        return Err(Error::<T>::InvalidEra.into());
      }

      let era_blocks_count = T::Fees::era_blocks_count();
      let cooldown_blocks_count = T::Sunrise::cooldown_blocks_count();
      let eras_in_cooldown = cooldown_blocks_count
        .checked_div(&era_blocks_count)
        .ok_or(Error::<T>::EraNotReady)?;
      let minimum_era_index_as_blocknumber =
        T::BlockNumber::from(current_era.index).saturating_sub(eras_in_cooldown);

      // Unable to claim previous era's if the `T::Cooldown` cooldown isnt cleared
      if T::BlockNumber::from(era_index) >= minimum_era_index_as_blocknumber {
        return Err(Error::<T>::EraNotReady.into());
      }

      // 3. Claim rewards
      T::Sunrise::try_claim_sunrise_rewards(&account_id, era_index)?;

      // Don't take tx fees on success
      Ok(Pays::No.into())
    }
  }
}
