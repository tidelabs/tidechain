#!/usr/bin/env bash

set -e

echo "*** Build node spec ***"
./target/release/tidechain build-spec --disable-default-bootnode --raw --chain hertel-staging > ./node/service/res/hertel_staging.json
./target/release/tidechain build-spec --disable-default-bootnode --raw --chain tidechain-staging > ./node/service/res/tidechain_staging.json
