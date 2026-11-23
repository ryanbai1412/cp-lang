#!/usr/bin/env bash
# editors/neovim/test/smoke_vim.sh [CPL]: verifies the Vim runtime files
# (filetype detection, commentstring, syntax groups) in headless Vim, then
# `cpl setup vim` and `cpl lsp` through the vim-lsp it installs, and runs
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
# Windows' Vim reads C:/... paths, not Git Bash's /tmp/...; its Vim folder is vimfiles.
vimdir=.vim
if command -v cygpath >/dev/null; then
    tmp="$(cygpath -m "$tmp")" here="$(cygpath -m "$here")" nvim_dir="$(cygpath -m "$nvim_dir")"
    root="$(cygpath -m "$root")" cpl="$(cygpath -m "$cpl")"
    [ -f "$cpl.exe" ] && cpl="$cpl.exe"
    vimdir=vimfiles
fi
cat >"$tmp/a.cp" <<'CP'
fn main():
    let n: Int = read()
    print(n + 1)
CP

"$vim_bin" --clean -Nu NONE -Es \
    -c "set rtp+=$nvim_dir | let g:cpl_path = '$cpl'" \
    -c "filetype plugin indent on" \
    -c "syntax on" \
    -c "edit $tmp/a.cp" \
    -c "if &filetype !=# 'cp' | cquit 2 | endif" \
    -c "if &commentstring !=# '// %s' | cquit 3 | endif" \
    -c "if &shiftwidth != 4 | cquit 4 | endif" \
    -c "if &formatprg !=# '$cpl fmt --stdin' | cquit 6 | endif" \
    -c "if !hlexists('cpKeyword') || !hlexists('cpBoolean') || !hlexists('cpType') | cquit 5 | endif" \
    -c "qall!"

# `gq` formats through `cpl fmt --stdin`.
printf 'let  x=1\nprint( x )\n' >"$tmp/b.cp"
"$vim_bin" --clean -Nu NONE -Es \
    -c "set rtp+=$nvim_dir | let g:cpl_path = '$cpl'" \
    -c "filetype plugin indent on" \
    -c "edit $tmp/b.cp" \
    -c "normal! gggqG" \
    -c "wqall"
printf 'let x = 1\nprint(x)\n' | cmp -s - "$tmp/b.cp" || { echo "vim smoke: gq wrote $(cat "$tmp/b.cp")" >&2; exit 7; }

# `:make` lists type errors and lints together through `cpl lint`.
printf 'let x: Int = "a"\nprint(y)\nvar s = 0\nprint(s)\n' >"$tmp/c.cp"
"$vim_bin" --clean -Nu NONE -Es \
    -c "set rtp+=$nvim_dir | let g:cpl_path = '$cpl'" \
    -c "filetype plugin indent on" \
    -c "edit $tmp/c.cp" \
    -c "silent make" \
    -c "if getqflist()->map({_, e -> [e.lnum, e.col, e.type]}) != [[1, 14, 'E'], [2, 7, 'E'], [3, 1, 'W']] | cquit 8 | endif" \
    -c "qall!" || { echo "vim smoke: :make did not list both errors and the lint" >&2; exit 8; }

# The documented setup: `cpl setup vim` writes a start package Vim loads.
"$cpl" setup vim --dir "$tmp/pack/cpl/start/cplang" >/dev/null
"$vim_bin" --clean -Nu NONE -Es \
    -c "set packpath^=$tmp" -c "packloadall" \
    -c "filetype plugin indent on" -c "syntax on" \
    -c "edit $tmp/a.cp" \
    -c "if &filetype !=# 'cp' || !exists('g:loaded_cplang') | cquit 6 | endif" \
    -c "qall!"

# ...with vim-lsp beside it, through which a wrong file gets `cpl lsp`'s error
# and completion its names (lsp_vim.vim). Vim exits when stdin ends, so it
# gets a pipe that stays open.
[ -f "$tmp/pack/cpl/start/vim-lsp/autoload/lsp.vim" ] || { echo "vim smoke: cpl setup vim did not install vim-lsp" >&2; exit 9; }
# A clean Vim: a home with an empty .vimrc (so no defaults.vim either) and
# only what `cpl setup vim` wrote under ~/.vim.
home="$tmp/home"
mkdir -p "$home/$vimdir"
cp -R "$tmp/pack" "$home/$vimdir/pack"
: >"$home/.vimrc"
printf 'fn main():\n    let total = 3\n    let y: Int = "text"\n    print(tot)\n' >"$tmp/bad.cp"
(cd "$tmp" && HOME="$home" USERPROFILE="$home" "$vim_bin" -i NONE --not-a-term \
    --cmd "let g:cpl_path = '$cpl' | let g:cpl_result = '$tmp/lsp'" \
    bad.cp -S "$here/lsp_vim.vim" < <(exec sleep 90 2>/dev/null) >/dev/null 2>&1) || true
[ "$(cat "$tmp/lsp" 2>/dev/null)" = ok ] || { echo "vim smoke: cpl lsp in vim-lsp: $(cat "$tmp/lsp" 2>/dev/null || echo 'no result')" >&2; exit 10; }

python3 "$root/editors/lsp-smoke.py" vim "$cpl" lsp
echo "vim smoke: ok"
