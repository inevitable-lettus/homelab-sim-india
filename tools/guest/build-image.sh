#!/usr/bin/env bash
# Builds the 32-bit Alpine root filesystem the terminal missions run on, using
# v86's own recipe at a pinned commit. Needs Docker, git, python3 and zstd.
# Output: $WORK/images/alpine-rootfs-flat and $WORK/images/alpine-fs.json
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/versions.env"
WORK="${WORK:-$HERE/.work}"
SRC="$WORK/v86"

mkdir -p "$WORK"
if [ ! -d "$SRC/.git" ]; then
  git clone --filter=blob:none https://github.com/copy/v86.git "$SRC"
fi
git -C "$SRC" fetch --quiet origin "$V86_COMMIT" || true
git -C "$SRC" checkout --quiet "$V86_COMMIT"
echo "v86 commit: $(git -C "$SRC" rev-parse HEAD)"

python3 -m venv "$WORK/venv"
"$WORK/venv/bin/pip" install --quiet "zstandard==$ZSTANDARD_VERSION"
export PATH="$WORK/venv/bin:$PATH"

DOCKERFILE="$SRC/tools/docker/alpine/Dockerfile"
git -C "$SRC" checkout --quiet -- tools/docker/alpine/Dockerfile
# Our base image and package set, and no demo script.
sed -i.bak \
  -e "s|^FROM .*|FROM docker.io/i386/alpine:$ALPINE_VERSION|" \
  -e "s/^ENV ADDPKGS=.*/ENV ADDPKGS=\"$GUEST_PACKAGES\"/" \
  -e '/hello.js/d' "$DOCKERFILE"
rm -f "$DOCKERFILE.bak"

rm -rf "$SRC/images"
bash "$SRC/tools/docker/alpine/build.sh"

rm -rf "$WORK/images"
mkdir -p "$WORK/images"
mv "$SRC/images/alpine-rootfs-flat" "$SRC/images/alpine-fs.json" "$WORK/images/"
echo "rootfs files: $(find "$WORK/images/alpine-rootfs-flat" -type f | wc -l | tr -d ' ')"
