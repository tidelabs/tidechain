# Oracle

A simple Security module for Tidechain.

## Overview

The Security module provides functionality to manage the security of the network.

- Generate unique ID to prevent replay attacks
- Increment blocks only of the status is enabled

### Dispatchable Functions

- `set_status` - Change the chain status

### Public Functions

- `is_chain_running` - Check if chain is running
- `get_current_block_count` - Get latest block
- `get_unique_id` - Get unique ID backed with a nonce for `who`
