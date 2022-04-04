# Asset Registry

A simple asset registry module for Tidechain.

## Overview

The Assets Registry module provides functionality for asset management of fungible asset classes.

- Register a new asset class in Tidechain
- Enable / Disable asset class in the ecosystem (Withdrawals and Swap)
- Get account balance for all registered assets

### Dispatchable Functions

- `register` - Register new asset on chain
- `set_status` - Update asset status

### Public Functions

- `get_account_balances` - Get the balances of `who` for all assets.
- `get_account_balance` - Get the asset `id` balance of `who`.
- `get_assets` - Get all assets.
