#!/usr/bin/env bash
# editors/vscode/smoke.sh [CPL]: compiles and runs the VS Code extension's
# grammar + unit test suite, verifies client.ts launches `cpl lsp`, and runs
# editors/lsp-smoke.py against the real `cpl lsp` binary.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
cpl="${1:-${CPL:-$root/target/debug/cpl}}"
[ -x "$cpl" ] || { echo "vscode smoke: build cpl first: cargo build -p cpl-cli" >&2; exit 1; }
[ -d "$here/node_modules" ] || { echo "vscode smoke: run npm ci in editors/vscode first" >&2; exit 1; }

(cd "$here" && npm test)
grep -q "\['lsp'\]" "$here/src/client.ts" || { echo "vscode smoke: src/client.ts does not default server.args to ['lsp']" >&2; exit 1; }
python3 "$root/editors/lsp-smoke.py" vscode "$cpl" lsp
echo "vscode smoke: ok"
