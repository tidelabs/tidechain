# Oracle

An Oracle module for Tidechain.

## Overview

The Oracle module provides to confirm swap request and match market maker orders.

- Match a swap request with a market maker order
- Cancel swap request and release the funds
- Update oracle account
- Disable swap for the ecosystem

### Dispatchable Functions

- `confirm_swap` - Oracle have confirmation and confirm the trade
- `cancel_swap` - Oracle cancel a swap request and release remaining funds
- `set_account_id` - Update oracle account ID
- `set_status` - Update oracle status
- `update_assets_value` - Oracle submit latest TDFY price for all assets.
- `add_market_maker` - Whitelist an account as a market maker
- `remove_market_maker` - Remove an account from the whitelist

### Public Functions

- `is_oracle_enabled` - Check if oracle is enabled
- `is_market_maker` - Check if an account is a market maker
- `add_new_swap_in_queue` - Add a new swap request to the queue
- `remove_swap_from_queue` - Remove a swap request from the queue
