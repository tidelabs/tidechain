#!/usr/bin/env bash

set -e

echo "*** Build node spec ***"
./target/release/tidefi-node build-spec --disable-default-bootnode --chain testnet > ./resources/testnet-spec.json
./target/release/tidefi-node build-spec --disable-default-bootnode --chain dev > ./resources/tidefi-spec.json
