#!/usr/bin/env bash
# The editor setups besides VS Code, each against the real `cpl lsp`.
#
#   editors/smoke.sh              keyword lists against the compiler, then Neovim,
#                                 Emacs and Helix when installed (others are skipped,
#                                 and say so)
#
# Needs target/debug/cpl (cargo build -p cpl-cli) or $CPL. $NVIM, $EMACS and $HX
# name the editors when they are not nvim, emacs and hx on the PATH.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cpl="${CPL:-$root/target/debug/cpl}"
[ -x "$cpl" ] || { echo "editors: build cpl first: cargo build -p cpl-cli" >&2; exit 1; }

echo "editors: keyword lists match crates/cpl-syntax/src/token.rs"
python3 - "$root" <<'PY'
import pathlib, re, sys
root = pathlib.Path(sys.argv[1])
token = (root / "crates/cpl-syntax/src/token.rs").read_text()
body = token.split("pub fn keyword(text: &str)")[1].split("_ => return")[0]
compiler = set(re.findall(r'"([a-z]+)" =>', body))
lists = {
    "editors/neovim/syntax/cp.vim": lambda s: " ".join(
        l.split(None, 3)[3] for l in s.splitlines() if re.match(r"syntax keyword cp(Keyword|Boolean) ", l)),
    "editors/emacs/cp-mode.el": lambda s: " ".join(
        re.search(r"\(defconst cp-%s\s+'\((.*?)\)\)" % n, s, re.S).group(1) for n in ("keywords", "constants")),
}
bad = False
for path, words in lists.items():
    file = root / path
    if not file.exists():
        continue
    got = set(re.findall(r"[a-z]+", words(file.read_text())))
    if got != compiler:
        bad = True
        print(f"  {path}: missing {sorted(compiler - got)}, extra {sorted(got - compiler)}", file=sys.stderr)
sys.exit(1 if bad else 0)
PY

nvim="${NVIM:-nvim}"
if command -v "$nvim" >/dev/null && "$nvim" --headless --clean -c 'if has("nvim-0.10") | qa! | else | cq | endif' >/dev/null 2>&1; then
    echo "editors: neovim"
    "$nvim" --headless --clean -l "$root/editors/neovim/test/smoke.lua" "$cpl" "$root"
else
    echo "editors: SKIPPED neovim (no nvim 0.10 or later; set \$NVIM)"
fi

emacs="${EMACS:-emacs}"
if command -v "$emacs" >/dev/null; then
    echo "editors: emacs"
    "$emacs" --batch -Q -l "$root/editors/emacs/smoke.el" 2>&1 | grep 'emacs smoke'
else
    echo "editors: SKIPPED emacs (set \$EMACS)"
fi

hx="${HX:-hx}"
if command -v "$hx" >/dev/null; then
    echo "editors: helix"
    config="$(mktemp -d)"
    mkdir -p "$config/helix"
    sed "s|^command = \"cpl\"|command = \"$cpl\"|" "$root/editors/helix/languages.toml" >"$config/helix/languages.toml"
    health="$(XDG_CONFIG_HOME="$config" "$hx" --health cp 2>&1 | sed 's/\x1b\[[0-9;]*m//g')"
    rm -rf "$config"
    grep -q "✓ $cpl" <<<"$health" || { echo "$health" >&2; echo "editors: helix does not find cpl lsp" >&2; exit 1; }
    echo "helix smoke: ok"
else
    echo "editors: SKIPPED helix (set \$HX)"
fi

