#!/usr/bin/env bash
# Phase 1 spike: builds a small 32-bit Alpine image for v86 and reports its size.
# Output goes to spikes/terminal/images/ (gitignored). Needs Docker running.
set -euo pipefail

V86_COMMIT="${V86_COMMIT:-master}"
PACKAGES="wireguard-tools openssh-client-default iproute2 curl nano"

HERE="$(cd "$(dirname "$0")" && pwd)"
SRC="$HERE/.v86"
OUT="$HERE/images"

if [ ! -d "$SRC/.git" ]; then
  git clone --filter=blob:none https://github.com/copy/v86.git "$SRC"
fi
git -C "$SRC" checkout --quiet "$V86_COMMIT"
echo "v86 commit: $(git -C "$SRC" rev-parse HEAD)"

python3 -m venv "$SRC/.venv"
"$SRC/.venv/bin/pip" install --quiet zstandard
export PATH="$SRC/.venv/bin:$PATH"

DOCKERFILE="$SRC/tools/docker/alpine/Dockerfile"
git -C "$SRC" checkout --quiet -- tools/docker/alpine/Dockerfile
sed -i.bak "s/^ENV ADDPKGS=.*/ENV ADDPKGS=\"$PACKAGES\"/" "$DOCKERFILE"
rm -f "$DOCKERFILE.bak"
sed -i.bak '/hello.js/d' "$DOCKERFILE"
rm -f "$DOCKERFILE.bak"

rm -rf "$SRC/images"
bash "$SRC/tools/docker/alpine/build.sh"

rm -rf "$OUT"
mkdir -p "$OUT"
mv "$SRC/images/alpine-rootfs-flat" "$SRC/images/alpine-fs.json" "$OUT/"

files="$(find "$OUT/alpine-rootfs-flat" -type f | wc -l | tr -d ' ')"
total="$(du -sk "$OUT/alpine-rootfs-flat" | cut -f1)"
echo "packages: $PACKAGES"
echo "rootfs files: $files"
echo "rootfs compressed on disk: ${total} KB"
echo "fs index: $(wc -c <"$OUT/alpine-fs.json" | tr -d ' ') bytes"
