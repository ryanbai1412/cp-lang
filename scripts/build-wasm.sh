#!/usr/bin/env bash
# Rule: handbook/guidelines/ci-checklist.md#cir22.
# Builds the compiler for the browser: crates/cpl-wasm as a WebAssembly
# module, with the JavaScript that loads it, into dist/wasm/ (not tracked).
#
#   scripts/build-wasm.sh            build into dist/wasm/
#   scripts/build-wasm.sh --smoke    build, then run the Node smoke test
#   scripts/build-wasm.sh --out DIR  build into DIR
#   scripts/build-wasm.sh --names DIR  cpl.wasm only, keeping function names
#                                    (for web/scripts/wasm-size.mjs --why)
#
# Needs the wasm32 target once: rustup target add wasm32-unknown-unknown
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$root/dist/wasm"
smoke=false
names=false
strip=true
while [ $# -gt 0 ]; do
    case "$1" in
        --smoke) smoke=true ;;
        --out) out="$(realpath -m "${2:?--out needs a directory}")"; shift ;;
        --names) names=true; out="$(realpath -m "${2:?--names needs a directory}")"; shift ;;
        -h | --help) sed -n '2,11s/^# \{0,1\}//p' "$0"; exit 0 ;;
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
# built at "s" and not "z": its frames are a program's recursion, and at "z" a
# program can nest about half as many calls on a browser's stack. "s" nests as
# deep as level 2 and is 66 kB smaller gzipped, at about 1.5x the run time
# (prime-count's slowest test: 1.07 s against 0.72 s). The rest is built at
# "z", ~205 kB smaller gzipped (~274 kB smaller base64) than "s".
cd "$root"
mkdir -p "$out"
built="$(cargo metadata --format-version 1 --no-deps | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')/$target/wasm/cpl_wasm.wasm"
# Two modules: cpl.wasm (check, lint, run, trace, the language server: what
# every editor on the site loads) and cpl-build.wasm (all of that plus the
# C++ back end, explain and stress), which the worker loads on the first
# build, explain or stress. A first visit downloads only the first.
# No wasm-opt: binaryen 132's -Oz output gzips 16 KB larger than rustc's,
# and an optional step made the shipped bytes depend on the machine.
build() {
    cargo build -p cpl-wasm --target "$target" --profile wasm \
        --config 'profile.wasm.inherits="release"' \
        --config 'profile.wasm.opt-level="z"' \
        --config 'profile.wasm.package.cpl-interp.opt-level="s"' \
        --config 'profile.wasm.lto=true' \
        --config 'profile.wasm.codegen-units=1' \
        --config 'profile.wasm.panic="abort"' \
        --config "profile.wasm.strip=$strip" "$@"
}
if $names; then
    # Its own target dir, so the shipped build is not relinked.
    export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target}/wasm-names"
    built="$CARGO_TARGET_DIR/$target/wasm/cpl_wasm.wasm"
    strip=false
    build --no-default-features
    cp "$built" "$out/cpl.wasm"
    exit 0
fi
build --no-default-features
cp "$built" "$out/cpl.wasm"
build
cp "$built" "$out/cpl-build.wasm"
cp "$root"/crates/cpl-wasm/js/* "$out/"

for file in "$out"/*; do
    printf '%8d bytes  %s\n' "$(wc -c <"$file")" "${file#"$root"/}"
done
if command -v gzip >/dev/null; then
    for wasm in cpl.wasm cpl-build.wasm; do
        printf '%8d bytes  %s, gzipped\n' "$(gzip -9c "$out/$wasm" | wc -c)" "$wasm"
    done
fi

if $smoke; then
    node "$root/crates/cpl-wasm/tests/smoke.mjs" "$out"
fi
