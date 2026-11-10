# Sourced by the other scripts. Puts Rust on PATH without shell-profile changes.
export PATH="$HOME/.cargo/bin:/opt/homebrew/opt/rustup/bin:$PATH"
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
export ROOT
cd "$ROOT"
