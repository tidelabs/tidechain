#!/usr/bin/env bash

set -e

echo "*** Build node spec ***"
./target/release/tidefi-node build-spec --disable-default-bootnode --dev > ./resources/tidefi-spec.json
