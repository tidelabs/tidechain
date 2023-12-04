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

use super::*;
use frame_support::{
  inherent::Vec,
  traits::{fungibles::InspectHold, Get, OnRuntimeUpgrade, StorageVersion},
  weights::Weight,
  BoundedVec,
};
use sp_std::vec;
use tidefi_primitives::{Hash, Swap, SwapStatus};

pub mod v1 {
  use sp_core::H256;

  use super::*;

  pub struct MigrationToV1<T>(sp_std::marker::PhantomData<T>);
  impl<T: Config> OnRuntimeUpgrade for MigrationToV1<T> {
    fn on_runtime_upgrade() -> Weight {
      if StorageVersion::get::<Pallet<T>>() == 0 {
        let mut reads = 1u64;
        let mut writes = 0u64;
        let mut invalid_swap_ids: Vec<H256> = vec![];

        AccountSwaps::<T>::translate::<BoundedVec<(Hash, SwapStatus), T::SwapLimitByAccount>, _>(
          |account_id, old_swaps_statuses| {
            // Read AccountSwaps map
            reads = reads.saturating_add(1);
            // Write AccountSwaps map
            writes = writes.saturating_add(1);

            let mut new_swaps_statuses =
              BoundedVec::<(Hash, SwapStatus), T::SwapLimitByAccount>::new();
            for (swap_id, swap_status) in old_swaps_statuses.iter() {
              // Read Swaps map
              reads = reads.saturating_add(1);
              match Swaps::<T>::get(swap_id) {
                Some(swap) => {
                  if *swap_status == SwapStatus::Pending {
                    match swap.amount_from.checked_sub(swap.amount_from_filled) {
                      Some(amount_from_left) => {
                        let balance_on_hold =
                          T::CurrencyTidefi::balance_on_hold(swap.token_from, &account_id);
                        match balance_on_hold.checked_sub(amount_from_left) {
                          // Keep swap in storage
                          Some(_) => new_swaps_statuses
                            .try_push((*swap_id, SwapStatus::Pending))
                            .unwrap(),
                          // Mark swap as invalid as not enough funds reserved
                          None => invalid_swap_ids.push(*swap_id),
                        }
                      }
                      None => invalid_swap_ids.push(*swap_id), // Mark swap as invalid as it is already fullfilled
                    }
                  } else {
                    // Keep swap in storage if its status is not pending
                    new_swaps_statuses
                      .try_push((*swap_id, swap_status.clone()))
                      .unwrap();
                  }
                }
                None => {} // Remove swap from account swaps as its detail is not found in storage
              }
            }

            if new_swaps_statuses.len() > 0 {
              Some(new_swaps_statuses)
            } else {
              None
            }
          },
        );

        Swaps::<T>::translate::<Swap<T::AccountId, T::BlockNumber>, _>(|swap_id, old_swap| {
          // Read Swaps map
          reads = reads.saturating_add(1);
          // Write Swaps map
          writes = writes.saturating_add(1);

          log!(info, "Invalid swap {:?} is deleted successfully", swap_id);

          if invalid_swap_ids.contains(&swap_id) {
            None
          } else {
            Some(old_swap)
          }
        });

        log!(info, "Oracle migration v1 applied successfully");

        StorageVersion::new(1).put::<Pallet<T>>();

        T::DbWeight::get().reads_writes(reads, writes)
      } else {
        log!(warn, "Oracle migration skipping v1, should be removed");
        T::DbWeight::get().reads(1)
      }
    }
  }
}
