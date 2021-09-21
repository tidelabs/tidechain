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

- Install `subkey`, `jq`
```bash
curl https://getsubstrate.io -sSf | bash -s --
brew install jq
```

- Generate node key using `subkey`
```bash
Alice_Node_Key=$(subkey generate --scheme Ed25519 --output-type Json | jq -r '.secretSeed')
```

- Run Alice's node

```bash
# Purge any chain data from previous runs
./target/release/polkadex-node purge-chain --base-path /tmp/alice --chain local

# Start Alice's node
./target/release/tidefi-node --base-path /tmp/alice \
  --chain dev \
  --alice \
  --port 30333 \
  --ws-port 9945 \
  --rpc-port 9933 \
  --node-key $Alice_Node_Key \
  --validator
```

```bash
2021-09-21 08:41:27 TiDeFi Node
2021-09-21 08:41:27 ✌️  version 1.0.0-6712dba-x86_64-macos
2021-09-21 08:41:27 ❤️  by Semantic Network <https://semantic-network.com>, 2017-2021
2021-09-21 08:41:27 📋 Chain specification: Development
2021-09-21 08:41:27 🏷 Node name: Alice
2021-09-21 08:41:27 👤 Role: AUTHORITY
2021-09-21 08:41:27 💾 Database: RocksDb at /tmp/alice/chains/tidefi_devnet/db/full
2021-09-21 08:41:27 ⛓  Native runtime: node-268 (tidefi-official-0.tx2.au10)
2021-09-21 08:41:27 🔨 Initializing Genesis block/state (state: 0xaa6e…f921, header-hash: 0x0c34…ce67)
2021-09-21 08:41:27 👴 Loading GRANDPA authority set from genesis on what appears to be first startup.
2021-09-21 08:41:28 ⏱  Loaded block-time = 3s from block 0x0c34a9a32a42c852c3cf3348e0da1c249381610ae0672a99332de19b30a8ce67
2021-09-21 08:41:28 👶 Creating empty BABE epoch changes on what appears to be first startup.
2021-09-21 08:41:28 Using default protocol ID "sup" because none is configured in the chain specs
2021-09-21 08:41:28 🏷 Local node identity is: 12D3KooWDTkjLrcEKPMkU8USQdAb4Qy2g3Rx6wysVeK4TVUgwbcB
2021-09-21 08:41:28 📦 Highest known block at #0
2021-09-21 08:41:28 〽️ Prometheus exporter started at 127.0.0.1:9615
2021-09-21 08:41:28 Listening for new connections on 127.0.0.1:9944.
2021-09-21 08:41:28 👶 Starting BABE Authorship worker
```

Local node identity is: `12D3KooWDTkjLrcEKPMkU8USQdAb4Qy2g3Rx6wysVeK4TVUgwbcB` shows the Peer ID that Bob will need when booting from Alice's node. This value was determined by the --node-key that was used to start Alice's node.

Now that Alice's node is up and running, Bob can join the network by bootstrapping from her node.
```bash
# Purge any chain data from previous runs
./target/release/polkadex-node purge-chain --base-path /tmp/alice --chain local

# Start Bob's node
./target/release/tidefi-node --base-path /tmp/bob \
  --chain dev \
  --bob \
  --port 30334 \
  --ws-port 9946 \
  --rpc-port 9934 \
  --validator
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWDTkjLrcEKPMkU8USQdAb4Qy2g3Rx6wysVeK4TVUgwbcB
```

If all is going well, after a few seconds, the nodes should peer together and start producing blocks. You should see some lines like the following in the console that started Alice node.

```bash
2021-09-21 08:46:22 TiDeFi Node
2021-09-21 08:41:27 ✌️  version 1.0.0-6712dba-x86_64-macos
2021-09-21 08:41:27 ❤️  by Semantic Network <https://semantic-network.com>, 2017-2021
2021-09-21 08:41:27 📋 Chain specification: Development
2021-09-21 08:46:22 🏷 Node name: Bob
2021-09-21 08:46:22 👤 Role: AUTHORITY
2021-09-21 08:46:22 💾 Database: RocksDb at /tmp/bob/chains/tidefi_devnet/db/full
2021-09-21 08:46:22 ⛓  Native runtime: node-268 (tidefi-official-0.tx2.au10)
2021-09-21 08:46:22 Using default protocol ID "sup" because none is configured in the chain specs
2021-09-21 08:46:22 🏷 Local node identity is: 12D3KooWNW9WAEi24EX4fCrifoczzp5cGtehRre5X9ie4Zs4gjZ4
2021-09-21 08:46:22 Could not load all certificates: Custom { kind: InvalidData, error: Custom { kind: InvalidData, error: BadDER } }
2021-09-21 08:46:23 📦 Highest known block at #7
2021-09-21 08:46:23 Listening for new connections on 127.0.0.1:9946.
2021-09-21 08:46:23 👶 Starting BABE Authorship worker
2021-09-21 08:46:24 ✨ Imported #24 (0x5057…8489)
2021-09-21 08:46:24 🔍 Discovered new external address for our node: /ip4/192.168.0.116/tcp/30334/p2p/12D3KooWNW9WAEi24EX4fCrifoczzp5cGtehRre5X9ie4Zs4gjZ4
2021-09-21 08:46:27 ✨ Imported #25 (0x8b4f…afc4)
2021-09-21 08:46:28 💤 Idle (1 peers), best: #25 (0x8b4f…afc4), finalized #23 (0x0c95…f3c8), ⬇ 2.3kiB/s ⬆ 0.8kiB/s
2021-09-21 08:46:30 ✨ Imported #26 (0xa86f…9a41)
2021-09-21 08:46:33 ✨ Imported #27 (0x03c5…4f37)
2021-09-21 08:46:33 💤 Idle (1 peers), best: #27 (0x03c5…4f37), finalized #24 (0x5057…8489), ⬇ 0.8kiB/s ⬆ 0.5kiB/s
2021-09-21 08:46:36 ✨ Imported #28 (0xb4cb…00c6)
```

## Using docker

The following commands will setup a local TiDeFi network made of 2 nodes. It's using the node key (0000000000000000000000000000000000000000000000000000000000000001). But you should generate your own node key using the subkey as the above.

```bash
docker build . -f cicd/node-dev.yml -t tidefi-node
docker-compose -f cicd/docker-compose.local.yml up --force-recreate
```

## Connecting to the nodes
The development node is a Substrate-based node, so you can interact with it using standard Substrate tools. The two provided RPC endpoints are:
- HTTP: `http://127.0.0.1:9933`
- WS: `ws://127.0.0.1:9944`
