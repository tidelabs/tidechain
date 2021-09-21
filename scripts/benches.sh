#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e
runtime="tidefi"
standard_args="--release --locked --features=runtime-benchmarks"

echo "[+] Running all benchmarks for $runtime"

# shellcheck disable=SC2086
cargo +nightly run $standard_args benchmark \
    --chain "dev" \
    --list |\
  tail -n+2 |\
  cut -d',' -f1 |\
  uniq | \
  grep -v frame_system > "${runtime}_pallets"

# For each pallet found in the previous command, run benches on each function
while read -r line; do
  pallet="$(echo "$line" | cut -d' ' -f1)";
  echo "Runtime: $runtime. Pallet: $pallet";
# shellcheck disable=SC2086
cargo +nightly run $standard_args -- benchmark \
  --chain="dev" \
  --steps=50 \
  --repeat=20 \
  --pallet="$pallet" \
  --extrinsic="*" \
  --execution=wasm \
  --wasm-execution=compiled \
  --heap-pages=4096 \
  --header=./file_header.txt \
  --output="./runtime/src/weights/${pallet/::/_}.rs"
done < "${runtime}_pallets"
rm "${runtime}_pallets"