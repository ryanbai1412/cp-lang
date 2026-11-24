#!/usr/bin/env bash
# Regenerates the files main keeps in sync with spec/ and skills/: the bundled
# skill pages, spec/conformance/COVERAGE.md, pnpm-lock.yaml, VS Code's name tables, and fixed entries
# dropped from spec/conformance/prelude-gaps.txt (a new gap still fails the gate). CI's
# regen job runs it on every main push and commits what changed, so a direct
# `git push` needs no local step. Prints the changed paths.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
py() { uv run --quiet python "$@"; }
# Each step runs even when another fails (the gate still reports the failure),
# so one broken generator never leaves every other file stale.
run() { local out; out=$("$@" 2>&1) || { printf '%s\n' "$out" | tail -20 >&2; echo "regen: failed: $*" >&2; }; }
[ ! -f scripts/skill-gen.py ] || run py scripts/skill-gen.py
[ ! -f scripts/specgate/coverage.py ] || run py scripts/spec-gate.py coverage --write
[ ! -f scripts/specgate/prelude.py ] || run py scripts/spec-gate.py prelude --write
! command -v pnpm >/dev/null || run pnpm install --lockfile-only --silent
! command -v node >/dev/null || run node editors/vscode/scripts/language-data.js
git status --porcelain -- skills spec/conformance pnpm-lock.yaml editors/vscode/src/languageData.ts editors/vscode/syntaxes | cut -c4-
