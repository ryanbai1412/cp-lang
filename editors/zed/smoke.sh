#!/usr/bin/env bash
# editors/zed/smoke.sh CPL: the Zed extension builds for Zed's wasm target, its grammar
# commit has editors/tree-sitter-cp as it is in the tree, and the server it starts
# (`cpl lsp`, as src/lib.rs returns it) answers `initialize` and publishes the error in a
# file (Zed has no headless mode, so editors/lsp-smoke.py stands in for it).
set -euo pipefail
cpl="$1"
here="$(cd "$(dirname "$0")" && pwd)"
root="$here/../.."
cargo build -q --release --target wasm32-wasip1 --manifest-path "$here/Cargo.toml" --target-dir "$here/target"
[ -f "$here/target/wasm32-wasip1/release/cp_lang_zed.wasm" ] || { echo "zed smoke: no extension.wasm" >&2; exit 1; }
rev="$(sed -n 's/^rev = "\(.*\)"$/\1/p' "$here/extension.toml")"
# A shallow clone (CI's checkout) lacks the commit; Zed fetches it from the repository too.
git -C "$root" cat-file -e "$rev^{commit}" 2>/dev/null || git -C "$root" fetch -q --depth 1 origin "$rev"
for file in grammar.js src/parser.c; do
    git -C "$root" cat-file -e "$rev:editors/tree-sitter-cp/$file" 2>/dev/null || { echo "zed smoke: grammar rev $rev has no editors/tree-sitter-cp/$file" >&2; exit 1; }
    git -C "$root" show "$rev:editors/tree-sitter-cp/$file" | cmp -s - "$root/editors/tree-sitter-cp/$file" ||
        { echo "zed smoke: editors/tree-sitter-cp/$file changed since $rev; point extension.toml's rev at the new commit" >&2; exit 1; }
done
grep -q 'vec!\["lsp".into()\]' "$here/src/lib.rs" || { echo "zed smoke: src/lib.rs does not start \`cpl lsp\`" >&2; exit 1; }
python3 "$root/editors/lsp-smoke.py" zed "$cpl" lsp
echo "zed smoke: ok"
