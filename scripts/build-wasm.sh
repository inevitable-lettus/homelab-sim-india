#!/bin/sh
set -eu
. "$(dirname -- "$0")/env.sh"

cargo build --release --target wasm32-unknown-unknown -p sim-wasm
wasm-bindgen --target web --out-dir app/src/wasm-pkg \
  target/wasm32-unknown-unknown/release/sim_wasm.wasm

# Shrink the module. The size budget in scripts/check.sh depends on this.
WASM_OPT=node_modules/.bin/wasm-opt
if [ ! -x "$WASM_OPT" ]; then
  echo "wasm-opt is missing. Run: npm ci" >&2
  exit 1
fi
"$WASM_OPT" -Oz app/src/wasm-pkg/sim_wasm_bg.wasm -o app/src/wasm-pkg/sim_wasm_bg.wasm
