#!/usr/bin/env bash
# `-p <crate>` for each workspace crate a change since $1 touches, for cargo; `--workspace`
# when $1 is no commit here or the change touches the workspace's own Cargo files.
set -euo pipefail
base=${1:-}
if [ -z "$base" ] || ! git rev-parse -q --verify "$base^{commit}" >/dev/null; then
    echo --workspace
    exit 0
fi
files=$(git diff --name-only "$base" HEAD)
if printf '%s\n' "$files" | grep -qxE 'Cargo\.(toml|lock)|rust-toolchain(\.toml)?|clippy\.toml'; then
    echo --workspace
    exit 0
fi
for dir in $(printf '%s\n' "$files" | sed -n 's|^crates/\([^/]*\)/.*|\1|p' | sort -u); do
    [ -f "crates/$dir/Cargo.toml" ] || continue
    name=$(sed -n 's/^name *= *"\([^"]*\)".*/\1/p' "crates/$dir/Cargo.toml" | head -1)
    [ -n "$name" ] && printf -- '-p %s ' "$name"
done
echo
