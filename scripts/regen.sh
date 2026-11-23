#!/usr/bin/env bash
# Regenerates the files main keeps in sync with spec/ and skills/: the bundled
# skill pages, spec/conformance/COVERAGE.md, and fixed entries dropped from
# spec/conformance/prelude-gaps.txt (a new gap still fails the gate). CI's
# regen job runs it on every main push and commits what changed, so a direct
# `git push` needs no local step. Prints the changed paths.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
py() { if command -v uv >/dev/null; then uv run --quiet python "$@"; else python3 "$@"; fi; }
[ -f scripts/skill-gen.py ] && py scripts/skill-gen.py >/dev/null
[ -f scripts/conformance-coverage.py ] && py scripts/conformance-coverage.py >/dev/null
if [ -f scripts/conformance-prelude.py ] && py scripts/conformance-prelude.py 2>&1 | grep -q ' 0 new,'; then
    py scripts/conformance-prelude.py --update >/dev/null
fi
git status --porcelain -- skills spec/conformance | cut -c4-
