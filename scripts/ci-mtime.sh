#!/usr/bin/env bash
# Makes a target/ restored from the Actions cache usable. Cargo decides what
# to rebuild by mtime, and a fresh checkout stamps every file with the
# checkout time, which would rebuild everything. This back-dates every
# tracked file, then re-stamps only the files that changed since the commit
# the cached target/ was built from (target/.ci-sha), so unchanged crates and
# test binaries are reused. With no cached target/ it does nothing.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
base=$(cat target/.ci-sha 2>/dev/null || true)
[ -n "$base" ] || { echo "ci-mtime: no cached target/; full build"; exit 0; }
if ! git cat-file -e "$base^{commit}" 2>/dev/null && ! git fetch -q --depth=1 origin "$base"; then
    echo "ci-mtime: cached target/ is from $base, which is gone; full build"
    exit 0
fi
git ls-files -z | xargs -0 touch -h -c -d @946684800
git diff -z --name-only "$base" HEAD | xargs -0 -r touch -h -c
echo "ci-mtime: target/ from ${base:0:9}; $(git diff --name-only "$base" HEAD | wc -l) file(s) changed since"
