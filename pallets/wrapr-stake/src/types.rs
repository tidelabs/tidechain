use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;

/// Container for borrow balance information
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct StakeSnapshot<Balance> {
  /// Principal balance (with accrued interest)
  pub principal: Balance,
  /// Initial balance
  pub initial_balance: Balance,
  /// Duration of the stake
  pub duration: u32,
}
