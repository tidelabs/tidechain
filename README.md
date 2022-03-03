# Tidechain

This repo contains runtimes for the Tidechain and Lagoon networks.
The README provides information about installing the `tidechain` binary.
For more specific guides, like how to be a validator, see the [Tidechain Wiki](#).

## Installation

If you just wish to run a Tidechain node without compiling it yourself, you may
either run the latest binary from our [releases](https://github.com/tide-labs/tidechain/releases) page.

## Building

### Install via Cargo

Make sure you have the support software installed from the **Build from Source** section
below this section.

If you want to install Tidechain in your PATH, you can do so with with:

```bash
cargo install --git https://github.com/tide-labs/tidechain --tag <version> tidechain --locked
```

### Build from Source

If you'd like to build from source, first install Rust. You may need to add Cargo's bin directory
to your PATH environment variable. Restarting your computer will do this for you automatically.

```bash
curl https://sh.rustup.rs -sSf | sh
```

If you already have Rust installed, make sure you're using the latest version by running:

```bash
rustup update
```

Once done, finish installing the support software:

```bash
sudo apt install build-essential git clang libclang-dev pkg-config libssl-dev
```

Build the client by cloning this repository and running the following commands from the root
directory of the repo:

```bash
git checkout <latest tagged release>
./scripts/init.sh
cargo build --release
```

Note that compilation is a memory intensive process. We recommend having 4 GiB of physical RAM or swap available (keep in mind that if a build hits swap it tends to be very slow).

#### Build from Source with Docker

You can also build from source using
[Tidechain CI docker image](https://hub.docker.com/r/tidelabs/tidechain-ci):

```bash
git checkout <latest tagged release>
docker run --rm -it -w /shellhere/tidechain \
                    -v $(pwd):/tidechain/tidechain \
                    tidelabs/tidechain-ci:latest cargo build --release
sudo chown -R $(id -u):$(id -g) target/
```

## Networks

This repo supports runtimes for Tidechain and Lagoon.

Tidechain is built on top of Substrate, a modular framework for blockchains.
One feature of Substrate is to allow for connection to different networks using a single executable and configuring it with a start-up flag.

### Tidechain Mainnet

Currently Tidechain is the default option when starting a node.
Connect to the global Tidechain Mainnet network by running:

```bash
tidechain
```

You can see your node on [telemetry] (set a custom name with `--name "my custom name"`).

[telemetry]: https://telemetry.tidefi.io/#list/Tidechain

### Connect to the Lagoon Testnet

Lagoon is the latest test network for Tidechain.
The tokens on this network are called TIDE and they purposefully hold no economic value.
The `sudo` pallet is enabled on this network allowing the core-team to debug the chain.

Connect to the global Lagoon testnet by running:

```bash
tidechain --chain=lagoon
```

You can see your node on [telemetry] (set a custom name with `--name "my custom name"`).

[telemetry]: https://telemetry.tidefi.io/#list/Lagoon

### Obtaining TIDEs

For Lagoon's TIDE tokens, see the faucet [instructions](#) on the Wiki.

## Hacking on Tidechain

If you'd actually like to hack on Tidechain, you can grab the source code and build it. Ensure you have
Rust and the support software installed. This script will install or update Rust and install the
required dependencies (this may take up to 30 minutes on Mac machines):

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast
```

Then, grab the Tidechain source code:

```bash
git clone https://github.com/tide-labs/tidechain.git
cd tidechain
```

Then build the code. You will need to build in release mode (`--release`) to start a network. Only
use debug mode for development (faster compile times for development and testing).

```bash
./scripts/init.sh   # Install WebAssembly. Update Rust
cargo build # Builds all native code
```

You can run the tests if you like:

```bash
cargo test --all
```

You can start a development chain with:

```bash
cargo run -- --dev
```

Detailed logs may be shown by running the node with the following environment variables set:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev
```

### Development

You can run a simple single-node development "network" on your machine by running:

```bash
tidechain --dev
```

### Local Two-node Testnet

If you want to see the multi-node consensus algorithm in action locally, then you can create a
local testnet. You'll need two terminals open. In one, run:

```bash
tidechain --chain=tidechain-local --alice -d /tmp/alice
```

And in the other, run:

```bash
tidechain --chain=tidechain-local --bob -d /tmp/bob --port 30334 --bootnodes '/ip4/127.0.0.1/tcp/30333/p2p/ALICE_BOOTNODE_ID_HERE'
```

Ensure you replace `ALICE_BOOTNODE_ID_HERE` with the node ID from the output of the first terminal.

### Using Docker

[Using Docker](docs/docker.md)

## Contributing

### Contributing Guidelines

[Contribution Guidelines](CONTRIBUTING.md)

## License

Tidechain is [GPL 3.0 licensed](LICENSE).
