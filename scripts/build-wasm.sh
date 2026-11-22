#!/usr/bin/env bash
# Builds the compiler for the browser: crates/cpl-wasm as a WebAssembly
# module, with the JavaScript that loads it, into dist/wasm/ (not tracked).
#
#   scripts/build-wasm.sh            build into dist/wasm/
#   scripts/build-wasm.sh --smoke    build, then run the Node smoke test
#   scripts/build-wasm.sh --out DIR  build into DIR
#
# Needs the wasm32 target once: rustup target add wasm32-unknown-unknown
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$root/dist/wasm"
smoke=false
while [ $# -gt 0 ]; do
    case "$1" in
        --smoke) smoke=true ;;
        --out) out="$(realpath -m "${2:?--out needs a directory}")"; shift ;;
        -h | --help) sed -n '2,9s/^# \{0,1\}//p' "$0"; exit 0 ;;
        *) echo "build-wasm: unknown argument \`$1\`" >&2; exit 2 ;;
    esac
    shift
done

target=wasm32-unknown-unknown
if ! rustc --print target-libdir --target "$target" 2>/dev/null | xargs test -d; then
    echo "build-wasm: the $target target is not installed; run: rustup target add $target" >&2
    exit 1
fi

# A profile of its own, set here so the workspace's Cargo.toml stays as it
# is. A panic ends the call as a trap that cpl.js reports. The interpreter is
# built at level 2 and not "z": its frames are a program's recursion, and at
# "z" a program can nest about half as many calls on a browser's stack. The
# rest is built at "z", ~205 kB smaller gzipped (~274 kB smaller base64) than "s".
cd "$root"
cargo build -p cpl-wasm --target "$target" --profile wasm \
    --config 'profile.wasm.inherits="release"' \
    --config 'profile.wasm.opt-level="z"' \
    --config 'profile.wasm.package.cpl-interp.opt-level=2' \
    --config 'profile.wasm.lto=true' \
    --config 'profile.wasm.panic="abort"' \
    --config 'profile.wasm.strip=true'

mkdir -p "$out"
built="$(cargo metadata --format-version 1 --no-deps | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')/$target/wasm/cpl_wasm.wasm"
if command -v wasm-opt >/dev/null; then
    wasm-opt -Os "$built" -o "$out/cpl.wasm"
else
    cp "$built" "$out/cpl.wasm"
fi
cp "$root"/crates/cpl-wasm/js/* "$out/"

for file in "$out"/*; do
    printf '%8d bytes  %s\n' "$(wc -c <"$file")" "${file#"$root"/}"
done
if command -v gzip >/dev/null; then
    printf '%8d bytes  cpl.wasm, gzipped\n' "$(gzip -9c "$out/cpl.wasm" | wc -c)"
fi

if $smoke; then
    node "$root/crates/cpl-wasm/tests/smoke.mjs" "$out"
fi
