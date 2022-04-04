# Tidefi Stake

A staking module for Tidechain.

## Overview

The Tidefi Stake module provides functionality to stake currency for a period of time and earn rewards
based on the swap fees collected on-chain.

- Generate unique ID to prevent replay attacks
- Increment blocks only of the status is enabled

### Dispatchable Functions

- `stake` - Stake `currency_id` for `amount` for `period`
- `unstake` - Unstake `stake_id`

### Public Functions

- `account_id` - Stake module account id
- `on_session_end` - Triger on session end
