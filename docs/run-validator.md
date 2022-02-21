---
title: Run a validator
---

This page will instruct you how to set up a validator node on the TideChain Public Sandbox.

## Requirements

The most common way for a beginner to run a validator is on a cloud server running Linux. You may choose whatever VPS provider you prefer, and whichever operating system you are comfortable with. For this guide we will be using **Ubuntu 20.04**, but the instructions should be similar for other platforms.

The transaction weights in TideChain were benchmarked on standard hardware. It is recommended that validators run at least the standard hardware in order to ensure they are able to process all blocks in time. The following are not minimum requirements but if you decide to run with less than this, you may experience performance issues.

### Standard Hardware

For the full details of the standard hardware please see [here](https://github.com/paritytech/substrate/pull/5848)

- **CPU** - Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz
- **Storage** - A NVMe solid state drive. Should be reasonably sized to deal with blockchain growth. Starting around 80GB - 160GB will be okay for the first six months of TiDeFi, but will need to be re-evaluated every six months.
- **Memory** - 64GB

The specs posted above are by no means the minimum specs that you could use when running a validator, however you should be aware that if you are using less you may need to toggle some extra optimizations in order to match up to other validators that are running the standard.

## Building and Installing the `Tidechain` binary

### Using a prebuilt

The nature of pre-built binaries means that they may not work on your particular architecture or Linux distribution. If you see an error like `cannot execute binary file: Exec format error` it likely means the binary is not compatible with your system. You will either need to compile the source code or use Docker.

Download the latest Tidechain binary within Ubuntu by running the following command.

```
curl -sL https://github.com/tide-labs/tidechain/releases/latest/download/tidechain && chmod +x tidechain && mv tidechain /usr/local/bin/
```

### Build from source

To build the `Tidechain` binary from the source-code, use the [release](https://github.com/tide-labs/tidechain/tree/release) branch and follow the instructions in the [README](../README.md#build-from-source).

## Synchronize Chain Data

You can synchronize your node by running the following commands if you do not want to start in validator mode right away:

```
tidechain --chain=hertel-local --bootnodes=/dns/a.bootnode.sandbox.tidefi.io/tcp/30333/p2p/12D3KooWEQmRfrvLbDcmxm8dGpFqkpeyUm5rLTY2SjKH9VJxE7rj --pruning=archive
```

The `--pruning=archive` flag is implied by the `--validator` flag, it is only required explicitly if you start your node without one of these two options. If you do not set your pruning to archive node, even when not running in validator mode, you will need to re-sync your database when you switch.

## Start the node in validator mode

Once your node is fully synced, stop the process by pressing Ctrl-C. At your terminal prompt, you will now start running the node.

```
tidechain --chain=hertel-local --bootnodes=/dns/a.bootnode.sandbox.tidefi.io/tcp/30333/p2p/12D3KooWEQmRfrvLbDcmxm8dGpFqkpeyUm5rLTY2SjKH9VJxE7rj --validator --name "Validator-Test"
```

You can give your validator any name that you like, but note that others will be able to see it and it will be included in the list of all servers using the same telemetry server. Since numerous people are using telemetry, it is recommended that you choose something likely to be unique.

##### Running a validator as a service

Prepare a `tidechain.service` file

```
sudo vi /etc/systemd/system/validator.service
```

```
[Unit]
Description=Tidechain Sandbox Validator Service
After=network-online.target
Wants=network-online.target

[Service]
User=ubuntu
Group=ubuntu
ExecStart=tidechain --chain=hertel-local --bootnodes=/dns/a.bootnode.sandbox.tidefi.io/tcp/30333/p2p/12D3KooWEQmRfrvLbDcmxm8dGpFqkpeyUm5rLTY2SjKH9VJxE7rj --validator --name 'Validator-Test'
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Run a validator as a service

```
sudo systemctl daemon-reload
sudo systemctl start tidechain
sudo systemctl status tidechain
```

### Generating the Session Keys

You need to tell the chain your Session keys by signing and submitting an extrinsic. This is what associates your validator node with your Controller account on Tidechain.

If you are on a remote server, it is easier to run this command on the same machine (while the node is running with the default HTTP RPC port configured):

```
curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys", "params":[]}' http://localhost:9933
```

The output will have a hex-encoded "result" field. The result is the concatenation of the four public keys. Save this result for a later step.

You can restart your node at this point.

### Submitting the `setKeys` Transaction

You need to tell the chain your Session keys by signing and submitting an extrinsic. This is what associates your validator with your Controller account.

Go to [Staking > Account Actions](#), and click "Session Key" on the bonding account you generated earlier. Enter the output from `author_rotateKeys` in the field and click "Set Session Key".

[to be completed]
