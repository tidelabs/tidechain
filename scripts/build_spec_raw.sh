#!/usr/bin/env bash

set -e

echo "*** Build node spec ***"
./target/release/tidechain build-spec --raw --chain ./node/service/res/lagoon_staging.json > ./node/service/res/lagoon.json
./target/release/tidechain build-spec --raw --chain ./node/service/res/tidechain_staging.json > ./node/service/res/tidechain.json
