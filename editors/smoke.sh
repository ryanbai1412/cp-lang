#!/usr/bin/env bash
# The editor setups besides VS Code, each against the real `cpl lsp`.
#
#   editors/smoke.sh              keyword lists against the compiler, then each
#                                 editor's own smoke test when it is installed
#                                 (Neovim, Emacs, Helix), the tree-sitter grammar
#                                 when the tree-sitter CLI is, and the JetBrains
#                                 plugin's build when java is (others are skipped,
#                                 and say so)
#
# Needs target/debug/cpl (cargo build -p cpl-cli) or $CPL. $NVIM, $EMACS, $HX and
# $TREE_SITTER name the tools when they are not nvim, emacs, hx and tree-sitter on
# the PATH.
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
    "editors/tree-sitter-cp/grammar.js": lambda s: " ".join(
        re.findall(r"'([a-z]+)'", re.search(r"boolean: _ =>(.*?)function_definition:", s, re.S).group(1))),
    "editors/jetbrains/src/main/kotlin/dev/cplang/ide/CpLexer.kt": lambda s: " ".join(
        re.findall(r'"([a-z]+)"', re.search(r"val KEYWORDS: Set<String> = setOf\((.*?)\)", s, re.S).group(1))),
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
    CPL="$cpl" "$emacs" --batch -Q -l "$root/editors/emacs/smoke.el" 2>&1 | grep 'emacs smoke'
else
    echo "editors: SKIPPED emacs (set \$EMACS)"
fi

if command -v "${HX:-hx}" >/dev/null; then
    echo "editors: helix"
    "$root/editors/helix/smoke.sh" "$cpl"
else
    echo "editors: SKIPPED helix (set \$HX)"
fi

# The committed parser is what grammar.js generates (with tree-sitter 0.25), every port
# and library parses without an error, and each editor's queries are the same patterns.
ts="${TREE_SITTER:-tree-sitter}"
if command -v "$ts" >/dev/null; then
    echo "editors: tree-sitter"
    grammar="$root/editors/tree-sitter-cp"
    work="$(mktemp -d)"
    cp "$grammar/grammar.js" "$grammar/tree-sitter.json" "$work/"
    (cd "$work" && "$ts" generate --abi 14 >/dev/null)
    diff -r "$work/src" "$grammar/src" >/dev/null || { echo "editors: editors/tree-sitter-cp/src is stale; run tree-sitter generate --abi 14 there" >&2; exit 1; }
    echo '{"parser-directories": ["'"$root/editors"'"]}' >"$work/config.json"
    find "$root/proto/ports" "$root/stdlib" -name '*.cp' >"$work/files"
    stat="$(cd "$grammar" && "$ts" parse --config-path "$work/config.json" --quiet --stat --paths "$work/files" 2>&1)"
    grep -q "failed parses: 0;" <<<"$stat" || { echo "$stat" >&2; echo "editors: the grammar fails on a .cp file" >&2; exit 1; }
    for query in "$grammar/queries/highlights.scm" "$root/editors/helix/runtime/queries/cp/highlights.scm" "$root/editors/zed/languages/cp/highlights.scm"; do
        [ -f "$query" ] || continue
        (cd "$grammar" && "$ts" query --config-path "$work/config.json" --quiet "$query" "$root/proto/ports/ecnerwala/cf2062g.cp" >/dev/null) || { echo "editors: $query does not compile" >&2; exit 1; }
    done
    python3 - "$root/editors" <<'PY'
import pathlib, re, sys
editors = pathlib.Path(sys.argv[1])
def patterns(path):
    text = "\n".join(l for l in path.read_text().splitlines() if not l.startswith(";"))
    return re.sub(r"@[a-z.]+", "@", text).strip()
want = patterns(editors / "tree-sitter-cp/queries/highlights.scm")
for path in [editors / "helix/runtime/queries/cp/highlights.scm", editors / "zed/languages/cp/highlights.scm"]:
    if path.exists() and patterns(path) != want:
        sys.exit(f"editors: {path} does not have the patterns of tree-sitter-cp/queries/highlights.scm")
PY
    rm -rf "$work"
    echo "tree-sitter smoke: ok"
else
    echo "editors: SKIPPED tree-sitter (set \$TREE_SITTER)"
fi

if command -v java >/dev/null; then
    echo "editors: jetbrains"
    (cd "$root/editors/jetbrains" && ./gradlew -q buildPlugin)
    echo "jetbrains build: ok"
else
    echo "editors: SKIPPED jetbrains (no java)"
fi
