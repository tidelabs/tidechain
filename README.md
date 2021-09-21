# Tidefi Substrate Node

Based on the substrate-node-template [Substrate](https://github.com/substrate-developer-hub/substrate-node-template) with custom pallets.

## Build the TiDeFi Node

To build TiDeFi node, you will need a proper Substrate development environment. If you need a refresher setting up your Substrate environment, see [Substrate's Getting Started Guide](https://substrate.dev/docs/en/knowledgebase/getting-started/).

Note that cloning master might result in an unstable build.

```bash
# Fetch the code
git clone https://tributary.semantic-network.tech/semnet/tidefi/back/tidefi-substrate-node.git
cd tidefi-substrate-node

# Build the node (The first build will be long (~30min))
cargo build --release
```

If a cargo not found error shows up in the terminal, manually add Rust to your system path (or restart your system):

```bash
source $HOME/.cargo/env
```

Then, you will want to run the node in dev mode using the following command:

```bash
./target/release/tidefi-node --dev
```

> For people not familiar with Substrate, the --dev flag is a way to run a Substrate-based node in a single node developer configuration for testing purposes. You can learn more about `--dev` in [this Substrate tutorial](https://substrate.dev/docs/en/tutorials/create-your-first-substrate-chain/interact).

When running a node via the binary file, data is stored in a local directory typically located in ~/.local/shared/tidefi-node/chains/development/db. If you want to start a fresh instance of the node, you can either delete the content of the folder, or run the following command inside the tidefi folder:

```bash
./target/release/node-tidefi purge-chain --dev
```

This will remove the data folder, note that all chain data is now lost.

## Run a local network (two nodes)

_to be completed_