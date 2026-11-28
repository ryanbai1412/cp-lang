#!/usr/bin/env bash
# Registers the merge drivers .gitattributes names, so a rebase never leaves conflict
# markers in a generated file. install-hooks.sh and land.sh run it; it is idempotent.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
git config merge.cpl-ratchet.name "debt-ratchet.json: lower of both sides (scripts/ratchet_merge.py)"
git config merge.cpl-ratchet.driver "python3 scripts/ratchet_merge.py --driver %O %A %B"
git config merge.cpl-regen.name "generated file: keep one side; land.sh regenerates it"
git config merge.cpl-regen.driver true
