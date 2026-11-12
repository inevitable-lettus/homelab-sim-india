#!/usr/bin/env bash
# Builds everything the in-browser terminal downloads, into app/public/guest/.
#   sh tools/guest/build.sh              build the root filesystem with Docker, then the state
#   GUEST_IMAGES=dir sh tools/guest/build.sh   reuse a root filesystem built earlier
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
. "$HERE/versions.env"
export WORK="${WORK:-$HERE/.work}"
OUT="$ROOT/app/public/guest"

if [ -z "${GUEST_IMAGES:-}" ]; then
  bash "$HERE/build-image.sh"
  GUEST_IMAGES="$WORK/images"
  GUEST_BIOS="$WORK/v86/bios"
fi
: "${GUEST_BIOS:?set GUEST_BIOS to a folder with seabios.bin and vgabios.bin}"

rm -rf "$OUT"
mkdir -p "$OUT"
cp -R "$GUEST_IMAGES/alpine-rootfs-flat" "$OUT/rootfs"
cp "$GUEST_IMAGES/alpine-fs.json" "$OUT/fs.json"
cp "$ROOT/node_modules/v86/build/libv86.mjs" "$ROOT/node_modules/v86/build/v86.wasm" "$OUT/"

GUEST_OUT="$OUT" GUEST_BIOS="$GUEST_BIOS" GUEST_MEMORY_MB="$GUEST_MEMORY_MB" V86_COMMIT="$V86_COMMIT" \
  ALPINE_VERSION="$ALPINE_VERSION" node "$HERE/build-state.mjs"

echo "guest files: $(find "$OUT" -type f | wc -l | tr -d ' '), $(du -sk "$OUT" | cut -f1) KB in $OUT"
