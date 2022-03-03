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

//! A set of constant values used in substrate runtime.

/// Money matters.
pub mod currency {
  use crate::{
    types::{AccountId, Balance, CurrencyId},
    Assets, Balances, DispatchError, DispatchResult,
  };
  use frame_support::traits::{
    fungible::{
      Inspect as FungibleInspect, InspectHold as FungibleInspectHold, Mutate as FungibleMutate,
      MutateHold as FungibleMutateHold, Transfer as FungibleTransfer,
    },
    fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
    tokens::{DepositConsequence, WithdrawConsequence},
  };
  use sp_std::marker::PhantomData;

  pub const TIDE: Balance = 1_000_000_000_000;
  pub const UNITS: Balance = TIDE;
  pub const DOLLARS: Balance = TIDE; // 10_000_000_000
  pub const CENTS: Balance = DOLLARS / 100; // 100_000_000
  pub const MILLICENTS: Balance = CENTS / 1_000; // 1_000_000

  pub const fn deposit(items: u32, bytes: u32) -> Balance {
    items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
  }

  pub struct Adapter<AccountId> {
    phantom: PhantomData<AccountId>,
  }

  impl Inspect<AccountId> for Adapter<AccountId> {
    type AssetId = CurrencyId;
    type Balance = Balance;

    fn total_issuance(asset: Self::AssetId) -> Self::Balance {
      match asset {
        CurrencyId::Tide => Balances::total_issuance(),
        CurrencyId::Wrapped(asset_id) => Assets::total_issuance(asset_id),
      }
    }

    fn balance(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
      match asset {
        CurrencyId::Tide => Balances::balance(who),
        CurrencyId::Wrapped(asset_id) => Assets::balance(asset_id, who),
      }
    }

    fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
      match asset {
        CurrencyId::Tide => Balances::minimum_balance(),
        CurrencyId::Wrapped(asset_id) => Assets::minimum_balance(asset_id),
      }
    }

    fn reducible_balance(asset: Self::AssetId, who: &AccountId, keep_alive: bool) -> Self::Balance {
      match asset {
        CurrencyId::Tide => Balances::reducible_balance(who, keep_alive),
        CurrencyId::Wrapped(asset_id) => Assets::reducible_balance(asset_id, who, keep_alive),
      }
    }

    fn can_deposit(
      asset: Self::AssetId,
      who: &AccountId,
      amount: Self::Balance,
    ) -> DepositConsequence {
      match asset {
        CurrencyId::Tide => Balances::can_deposit(who, amount),
        CurrencyId::Wrapped(asset_id) => Assets::can_deposit(asset_id, who, amount),
      }
    }

    fn can_withdraw(
      asset: Self::AssetId,
      who: &AccountId,
      amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
      match asset {
        CurrencyId::Tide => Balances::can_withdraw(who, amount),
        CurrencyId::Wrapped(asset_id) => Assets::can_withdraw(asset_id, who, amount),
      }
    }
  }

  impl InspectHold<AccountId> for Adapter<AccountId> {
    fn balance_on_hold(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
      match asset {
        CurrencyId::Tide => Balances::balance_on_hold(who),
        CurrencyId::Wrapped(asset_id) => Assets::balance_on_hold(asset_id, who),
      }
    }
    fn can_hold(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> bool {
      match asset {
        CurrencyId::Tide => Balances::can_hold(who, amount),
        CurrencyId::Wrapped(asset_id) => Assets::can_hold(asset_id, who, amount),
      }
    }
  }

  impl MutateHold<AccountId> for Adapter<AccountId> {
    fn hold(asset: CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
      match asset {
        CurrencyId::Tide => Balances::hold(who, amount),
        CurrencyId::Wrapped(asset_id) => Assets::hold(asset_id, who, amount),
      }
    }

    fn release(
      asset: CurrencyId,
      who: &AccountId,
      amount: Balance,
      best_effort: bool,
    ) -> Result<Balance, DispatchError> {
      match asset {
        CurrencyId::Tide => Balances::release(who, amount, best_effort),
        CurrencyId::Wrapped(asset_id) => Assets::release(asset_id, who, amount, best_effort),
      }
    }
    fn transfer_held(
      asset: CurrencyId,
      source: &AccountId,
      dest: &AccountId,
      amount: Balance,
      best_effort: bool,
      on_hold: bool,
    ) -> Result<Balance, DispatchError> {
      match asset {
        CurrencyId::Tide => Balances::transfer_held(source, dest, amount, best_effort, on_hold),
        CurrencyId::Wrapped(asset_id) => {
          Assets::transfer_held(asset_id, source, dest, amount, best_effort, on_hold)
        }
      }
    }
  }

  impl Mutate<AccountId> for Adapter<AccountId> {
    fn mint_into(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
      match asset {
        CurrencyId::Tide => Balances::mint_into(who, amount),
        CurrencyId::Wrapped(asset_id) => Assets::mint_into(asset_id, who, amount),
      }
    }

    fn burn_from(
      asset: Self::AssetId,
      who: &AccountId,
      amount: Balance,
    ) -> Result<Balance, DispatchError> {
      match asset {
        CurrencyId::Tide => Balances::burn_from(who, amount),
        CurrencyId::Wrapped(asset_id) => Assets::burn_from(asset_id, who, amount),
      }
    }
  }

  impl Transfer<AccountId> for Adapter<AccountId>
  where
    Assets: Transfer<AccountId>,
  {
    fn transfer(
      asset: Self::AssetId,
      source: &AccountId,
      dest: &AccountId,
      amount: Self::Balance,
      keep_alive: bool,
    ) -> Result<Balance, DispatchError> {
      match asset {
        CurrencyId::Tide => {
          <Balances as FungibleTransfer<AccountId>>::transfer(source, dest, amount, keep_alive)
        }
        CurrencyId::Wrapped(asset_id) => {
          <Assets as Transfer<AccountId>>::transfer(asset_id, source, dest, amount, keep_alive)
        }
      }
    }
  }
}

pub mod time {
  use crate::types::{BlockNumber, Moment};

  /// Since BABE is probabilistic this is the average expected block time that
  /// we are targeting. Blocks will be produced at a minimum duration defined
  /// by `SLOT_DURATION`, but some slots will not be allocated to any
  /// authority and hence no block will be produced. We expect to have this
  /// block time on average following the defined slot duration and the value
  /// of `c` configured for BABE (where `1 - c` represents the probability of
  /// a slot being empty).
  /// This value is only used indirectly to define the unit constants below
  /// that are expressed in blocks. The rest of the code should use
  /// `SLOT_DURATION` instead (like the Timestamp pallet for calculating the
  /// minimum period).
  ///
  /// If using BABE with secondary slots (default) then all of the slots will
  /// always be assigned, in which case `MILLISECS_PER_BLOCK` and
  /// `SLOT_DURATION` should have the same value.
  ///
  /// <https://research.web3.foundation/en/latest/polkadot/block-production/Babe.html#-6.-practical-results>
  pub const MILLISECS_PER_BLOCK: Moment = 6000;
  pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;

  // NOTE: Currently it is not possible to change the slot duration after the chain has started.
  // Attempting to do so will brick block production.
  pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

  // 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
  pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

  // NOTE: Currently it is not possible to change the epoch duration after the chain has started.
  // Attempting to do so will brick block production.
  pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 4 * HOURS;
  pub const EPOCH_DURATION_IN_SLOTS: u32 = {
    const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

    (EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u32
  };

  // These time units are defined in number of blocks.
  pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
  pub const HOURS: BlockNumber = MINUTES * 60;
  pub const DAYS: BlockNumber = HOURS * 24;
}

/// Fee-related.
pub mod fee {
  pub use sp_runtime::Perbill;

  use crate::types::{Balance, ExtrinsicBaseWeight};
  use frame_support::weights::{
    WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
  };
  use smallvec::smallvec;

  /// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
  /// node's balance type.
  ///
  /// This should typically create a mapping between the following ranges:
  ///   - [0, MAXIMUM_BLOCK_WEIGHT]
  ///   - [Balance::min, Balance::max]
  ///
  /// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
  ///   - Setting it to `0` will essentially disable the weight fee.
  ///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
  pub struct WeightToFee;
  impl WeightToFeePolynomial for WeightToFee {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
      // in Tidechain, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
      let p = super::currency::CENTS;
      let q = 10 * Balance::from(ExtrinsicBaseWeight::get());
      smallvec![WeightToFeeCoefficient {
        degree: 1,
        negative: false,
        coeff_frac: Perbill::from_rational(p % q, q),
        coeff_integer: p / q,
      }]
    }
  }
}
