#!/usr/bin/env bash
# editors/neovim/test/smoke_vim.sh [CPL]: verifies the Vim runtime files
# (filetype detection, commentstring, syntax groups) in headless Vim and runs
# editors/lsp-smoke.py against the real `cpl lsp` binary.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
nvim_dir="$(cd "$here/.." && pwd)"
root="$(cd "$nvim_dir/../.." && pwd)"
cpl="${1:-${CPL:-$root/target/debug/cpl}}"
vim_bin="${VIM_BIN:-vim}"
[ -x "$cpl" ] || { echo "vim smoke: build cpl first: cargo build -p cpl-cli" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cat >"$tmp/a.cp" <<'CP'
fn main():
    let n: Int = read()
    print(n + 1)
CP

"$vim_bin" --clean -Nu NONE -Es \
    -c "set rtp+=$nvim_dir" \
    -c "filetype plugin indent on" \
    -c "syntax on" \
    -c "edit $tmp/a.cp" \
    -c "if &filetype !=# 'cp' | cquit 2 | endif" \
    -c "if &commentstring !=# '// %s' | cquit 3 | endif" \
    -c "if &shiftwidth != 4 | cquit 4 | endif" \
    -c "if !hlexists('cpKeyword') || !hlexists('cpBoolean') || !hlexists('cpType') | cquit 5 | endif" \
    -c "qall!"

python3 "$root/editors/lsp-smoke.py" vim "$cpl" lsp
echo "vim smoke: ok"
