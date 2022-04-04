# Fees

A Fees module for Tidechain.

## Overview

The Fees module provides functionality for swap fees collection and rewards distribution via sunrise pool and staking.

- Collect fees for each swap and accumulate the rewards
- Redistribute the rewards to the sunrise pool
- Redistribute the rewards to the staking module
- Claim sunrise rewards

### Dispatchable Functions

- `claim_sunrise_rewards` - Claim available sunrise rewards of `signer`

### Public Functions

- `start_era` - Initialze fee `era`.
- `account_id` - Get the account ID of the `Fees` pallet where the funds are stored.
- `calculate_swap_fees` - Calculate swap fee for the `currency_id` and `amount`
- `register_swap_fees` - Register swap fee for the `currency_id` and `amount` and the `account_id`
