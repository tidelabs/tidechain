# Lagoon Testnet

Lagoon is the latest test network for Tidechain.
The tokens on this network are called TDFY and they purposefully hold no economic value.
The `sudo` pallet is enabled on this network allowing the core-team to debug the chain.

Connect to the global Lagoon testnet by running:

```bash
tidechain --chain=lagoon
```

You can see your node on [telemetry] (set a custom name with `--name "my custom name"`).

[telemetry]: https://telemetry.tidefi.io/#list/Lagoon

## Chainspec

A chain specification, or "chain spec", is a collection of configuration information that dictates which network a blockchain node will connect to, which entities it will initially communicate with, and what consensus-critical state it must have at genesis.

## Local Development

This is used specially for local development (only 1 Validator and Sudo enabled)

```bash
tidechain --chain=lagoon-dev
```

- Validators: `Alice`
- Quorum: `Charlie`
- Oracle: `Ferdie`
- Root: `Ferdie`

## Devnet

This is the SEMNET development network, where the chain is updated with nightly changes. Available for the core-team only.

```bash
tidechain --chain=lagoon-local
```

- Validators: `Alice, Bob`
- Quorums: `Charlie, Dave, Eve`
- Oracle: `Ferdie`
- Root: `Ferdie`

## Testnet

This is used specially for local development (only 1 Validator and Sudo enabled)

```bash
tidechain --chain=lagoon
```

Validators:

```
5EPH4xUAqCbTPw3kfjqewELdrCEQiNp84vuhiVg8hU1A5wX7
5Gn8KDvn6ZfcqG4s5WBLbH4bARS77nFrA738tHaNrGkESUb9
5Ge8JkHNACxSVR9vNpirDmLNHBVCPqhybEe61BSKsgimEEgr
```

Quorums:

```
5EFKNPG2kPsyeVK8E5e7i5uiRfYdbQkq8qfhVxeVV42tZfPe
5HVb1QTxnzHXpTPLCVT61Ag3Mb4fmyMYAy3kxbXYXMS9KjM6
5EA2mLbbbdq6cyqDwZuHEGvKPPBVWDNuCS3DwtaetAum9aSe
```

Oracle: `5HKDZMoz5NnX37Np8dMKMAANbNu9N1XuQec15b3tZ8NaBTAR`

Root: `5Hp9T9DoHRmLXsZ6j85R7xxqmUxCZ7MS4pfi4C6W6og484G6`
