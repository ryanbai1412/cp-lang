#!/usr/bin/env bash
# One fresh full cpl-next run on this checkout's HEAD, kept for scripts/hourly.py under
# target/cpl-next/fresh/<sha>/ (the run's own verdict files, never the committed lists).
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
sha=$(git rev-parse --short=9 HEAD)
out=target/cpl-next/fresh/$sha
mkdir -p "$out"
cargo test -q -p cpl-next --test programs every_case_is_where_the_lists_say >"$out/run.log" 2>&1
for f in passing.txt wrong.tsv refused.tsv masked.txt error-typed.txt lowered.txt; do
    [ -f "target/cpl-next/$f" ] && cp "target/cpl-next/$f" "$out/"
done
date +%s >"$out/stamp"
grep -E '^cpl-next: ' "$out/run.log" || tail -3 "$out/run.log"
