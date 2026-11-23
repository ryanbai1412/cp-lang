#!/usr/bin/env bash
# editors/helix/smoke.sh CPL: Helix with editors/helix as its whole configuration builds
# the grammar, finds the server and the highlight queries, and the server it starts
# answers `initialize` and publishes the error in a file (Helix has no headless mode,
# so editors/lsp-smoke.py starts the command languages.toml names).
set -euo pipefail
cpl="$1"
here="$(cd "$(dirname "$0")" && pwd)"
hx="${HX:-hx}"
config="$(mktemp -d)"
trap 'rm -rf "$config"' EXIT
mkdir -p "$config/helix/runtime/queries" "$config/helix/runtime/grammars"
{ echo 'use-grammars = { only = ["cp"] }'; sed -e "s|^command = \"cpl\"|command = \"$cpl\"|" \
    -e "s|/path/to/cp-lang/editors/tree-sitter-cp|$here/../tree-sitter-cp|" \
    "$here/languages.toml"; } >"$config/helix/languages.toml"
ln -s "$here/runtime/queries/cp" "$config/helix/runtime/queries/cp"
XDG_CONFIG_HOME="$config" "$hx" --grammar build >/dev/null 2>&1 || {
    XDG_CONFIG_HOME="$config" "$hx" --grammar build >&2
    echo "helix smoke: the grammar does not build" >&2
    exit 1
}
health="$(XDG_CONFIG_HOME="$config" "$hx" --health cp 2>&1 | sed 's/\x1b\[[0-9;]*m//g')"
# Helix 25.07 names the server before its path (`✓ cpl: PATH`); older ones print `✓ PATH`.
grep -qE "✓ (cpl: )?$cpl\$" <<<"$health" || { echo "$health" >&2; echo "helix smoke: the server \`$cpl\` is not found" >&2; exit 1; }
for want in "Tree-sitter parser: ✓" "Highlight queries: ✓"; do
    grep -qF "$want" <<<"$health" || { echo "$health" >&2; echo "helix smoke: no \`$want\`" >&2; exit 1; }
done
mapfile -t command < <(python3 - "$config/helix/languages.toml" <<'PY'
import re, sys
section = open(sys.argv[1]).read().split("[language-server.cpl]")[1].split("\n[")[0]
command = re.search(r'^command = "(.*)"$', section, re.M).group(1)
args = re.findall(r'"([^"]*)"', re.search(r"^args = \[(.*)\]$", section, re.M).group(1))
print(command, *args, sep="\n")
PY
)
python3 "$here/../lsp-smoke.py" helix "${command[@]}"
echo "helix smoke: ok"
