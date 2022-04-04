# Quorum

The Quorum module for Tidechain.

## Overview

The Quorum handles voting and execution of proposals, administration
of the members set and signaling transfers.

### Dispatchable Functions

- `submit_proposal` - Quorum member submit proposal
- `acknowledge_proposal` - Quorum member acknowledge proposal
- `reject_proposal` - Quorum member reject proposal
- `acknowledge_burned` - Quorum member acknowledge burned proposal and initiated the process
- `eval_proposal_state` - Evaluate the state of a proposal given the current vote threshold
- `submit_public_keys` - Quorum member submit his own public keys for all chains

### Public Functions

- `is_quorum_enabled` - Check if quorum is enabled
- `add_new_withdrawal_in_queue` - Add a new withdrawal request to the queue
