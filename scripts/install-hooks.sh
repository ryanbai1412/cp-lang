#!/usr/bin/env bash
# Points this clone's git hooks at .githooks/, so every commit runs
# scripts/check.sh --fast first. Run it once after cloning.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
git config core.hooksPath .githooks
scripts/merge-drivers.sh
echo "hooks installed: git commit runs scripts/check.sh --fast + commit-msg, then post-commit starts a background wasm build; git push runs .githooks/pre-push"
