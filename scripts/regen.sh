#!/usr/bin/env bash
# Regenerates the files main keeps in sync with spec/ and skills/: every gen.toml
# entry, pnpm-lock.yaml and VS Code's name tables. CI's
# regen job runs it on every main push and commits what changed, so a direct
# `git push` needs no local step. Prints the changed paths.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
py() { uv run --quiet python "$@"; }
# Each step runs even when another fails (the gate still reports the failure),
# so one broken generator never leaves every other file stale.
run() { local out; out=$("$@" 2>&1) || { printf '%s\n' "$out" | tail -20 >&2; echo "regen: failed: $*" >&2; }; }
# Every gen.toml entry, then its check: a writer that fails, or a file still
# stale after its writer ran, turns the regen job red.
run py scripts/gen.py
run py scripts/gen.py --check --ci
! command -v pnpm >/dev/null || run pnpm install --lockfile-only --silent
! command -v node >/dev/null || run node editors/vscode/scripts/language-data.js
git status --porcelain --untracked-files=all | cut -c4-
