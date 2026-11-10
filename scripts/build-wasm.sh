#!/bin/sh
set -eu
. "$(dirname -- "$0")/env.sh"

cargo build --release --target wasm32-unknown-unknown -p sim-wasm
wasm-bindgen --target web --out-dir app/src/wasm-pkg \
  target/wasm32-unknown-unknown/release/sim_wasm.wasm
