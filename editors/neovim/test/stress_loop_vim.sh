#!/usr/bin/env bash
# editors/neovim/test/stress_loop_vim.sh [CPL]: the debugger's wrong-answer loop
# from plain Vim as editors/README.md documents it: `:!cpl stress %` finds and
# saves a small failing case of the planted knapsack, `:!cpl test %` fails on
# it, the fix is written, and `:!cpl test %` and `:!cpl stress %` pass.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
cpl="${1:-${CPL:-$root/target/debug/cpl}}"
vim_bin="${VIM_BIN:-vim}"
[ -x "$cpl" ] || { echo "vim stress loop: build cpl first: cargo build -p cpl-cli" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir "$tmp/bin" && ln -s "$cpl" "$tmp/bin/cpl"
cp "$root/examples/debugging/knapsack.cp" "$tmp/k.cp"

cat >"$tmp/loop.vim" <<VIM
edit $tmp/k.cp
silent !cpl stress % >/dev/null 2>&1
if v:shell_error != 1 | cquit 2 | endif
if empty(glob('$tmp/k.tests/stress-*.in')) | cquit 3 | endif
silent !cpl test % >/dev/null 2>&1
if v:shell_error == 0 | cquit 4 | endif
%s/for c in 0\\.\\.=cap:/for c in (0..=cap).rev:/
write
silent !cpl test % >/dev/null 2>&1
if v:shell_error != 0 | cquit 5 | endif
silent !cpl stress % >/dev/null 2>&1
if v:shell_error != 0 | cquit 6 | endif
qall!
VIM
PATH="$tmp/bin:$PATH" "$vim_bin" --clean -Nu NONE -Es -S "$tmp/loop.vim" \
    || { echo "vim stress loop: failed with $?" >&2; exit 1; }
echo "vim stress loop: ok"
