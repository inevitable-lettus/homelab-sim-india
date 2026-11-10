#!/bin/sh
# The one command that must pass before any commit. CI runs exactly this.
set -eu
. "$(dirname -- "$0")/env.sh"

BUDGET_BYTES=512000

step() { printf '\n==> %s\n' "$1"; }

step "rust format"
cargo fmt --all --check
step "rust lint"
cargo clippy --workspace --all-targets --locked -- -D warnings
step "rust tests"
cargo test --workspace --locked

step "wasm and data"
sh scripts/build-wasm.sh
sh scripts/build-data.sh

step "app lint"
npm run lint -w app
step "app types"
npm run typecheck -w app
step "app tests"
npm run test -w app
step "app build"
npm run build -w app

step "size budget (gzipped, Architecture.md)"
total=0
for file in $(find app/dist/assets -type f \( -name '*.js' -o -name '*.css' -o -name '*.wasm' \)); do
  size="$(gzip -9 -c "$file" | wc -c | tr -d ' ')"
  printf '%10s  %s\n' "$size" "$file"
  total=$((total + size))
done
printf '%10s  total (budget %s)\n' "$total" "$BUDGET_BYTES"
if [ "$total" -gt "$BUDGET_BYTES" ]; then
  echo "size budget exceeded" >&2
  exit 1
fi

printf '\nAll checks passed.\n'
