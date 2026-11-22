#!/usr/bin/env bash
# Builds cpl for one release target with CPL_CHANNEL and CPL_COMMIT stamped
# in, and packs it into <out dir> with package.py. Linux targets link musl
# statically through cargo-zigbuild.
#
#   CPL_CHANNEL=edge CPL_COMMIT=<sha> scripts/release/build.sh <target> <out dir>
set -euo pipefail
cd "$(dirname "$0")/../.."
target="${1:?usage: build.sh <target> <out dir>}"
out="${2:?usage: build.sh <target> <out dir>}"
: "${CPL_CHANNEL:?set CPL_CHANNEL to stable or edge}"
: "${CPL_COMMIT:?set CPL_COMMIT to the commit being built}"
export CPL_CHANNEL CPL_COMMIT
export SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}"
rustup target add "$target" >/dev/null
case "$target" in
    *-linux-musl) cargo zigbuild --release --locked -p cpl-cli --target "$target" ;;
    *) cargo build --release --locked -p cpl-cli --target "$target" ;;
esac
exe=cpl
case "$target" in *windows*) exe=cpl.exe ;; esac
bin="${CARGO_TARGET_DIR:-target}/$target/release/$exe"
python3 scripts/release/package.py "$target" "$bin" "$out"
