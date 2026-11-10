#!/bin/sh
set -eu
. "$(dirname -- "$0")/env.sh"

cargo run --quiet -p data-build -- app/public/data/bundle.json
