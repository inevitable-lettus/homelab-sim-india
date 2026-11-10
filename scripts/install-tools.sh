#!/bin/sh
# Installs the wasm-bindgen CLI at the exact version pinned in Cargo.toml.
set -eu
. "$(dirname -- "$0")/env.sh"

version="$(sed -n 's/^wasm-bindgen = "=\(.*\)"$/\1/p' Cargo.toml)"
if [ -z "$version" ]; then
  echo "could not read the wasm-bindgen pin from Cargo.toml" >&2
  exit 1
fi

if command -v wasm-bindgen >/dev/null 2>&1 && [ "$(wasm-bindgen --version)" = "wasm-bindgen $version" ]; then
  echo "wasm-bindgen $version already installed"
  exit 0
fi

cargo install wasm-bindgen-cli --locked --version "$version" --force
