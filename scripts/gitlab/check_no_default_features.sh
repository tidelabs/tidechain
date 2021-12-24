#!/usr/bin/env bash

set -e

pushd node && cargo check --no-default-features && popd
