# Tidefi Substrate Node

Based on the substrate-node-template [Substrate](https://github.com/substrate-developer-hub/substrate-node-template) with the addition of [pallet_multisig](https://crates.io/crates/pallet-multisig) and [pallet contracts](https://crates.io/crates/pallet-contracts) :rocket:

### run it in on a server

`cargo build --release`

`./target/release/tidefi-substrate-node --dev --ws-external`

port 9944 needs to be open

### run it in temp mode

`cargo build --release`

`./target/release/tidefi-substrate-node --dev --tmp`

[playground](https://polkadot.js.org/apps/#/accounts)

### apps frontend

```
git checkout tags/v0.82.1 -b apps0.82.1

git reset --hard

yarn

yarn run start
```

0x69ee4dc4736938360b8f37a4d0ea5d63e70b9880945c4b81f641fb70f1c6aa08
