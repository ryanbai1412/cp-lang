#!/usr/bin/env bash
# Every editor setup in editors/, each against the real `cpl lsp`.
#
#   editors/smoke.sh              keyword lists against the compiler, then each
#                                 editor's own smoke test when its toolchain is
#                                 installed (VS Code, Neovim, Vim, Emacs, Helix,
#                                 Zed), the tree-sitter grammar when the
#                                 tree-sitter CLI is, and the JetBrains plugin's
#                                 build when java is (others are skipped, and say so)
#
# Needs target/debug/cpl (cargo build -p cpl-cli) or $CPL. $NVIM, $VIM_BIN,
# $EMACS, $HX and $TREE_SITTER name the tools when they are not on the PATH.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cpl="${CPL:-$root/target/debug/cpl}"
[ -x "$cpl" ] || { echo "editors: build cpl first: cargo build -p cpl-cli" >&2; exit 1; }

echo "editors: editors/words.json is crates/cpl-syntax/src/token.rs's tables"
(cd "$root" && uv run --quiet python scripts/editor-words.py --check)

if command -v node >/dev/null && command -v npm >/dev/null && [ -d "$root/editors/vscode/node_modules" ]; then
    echo "editors: vscode"
    "$root/editors/vscode/smoke.sh" "$cpl"
else
    echo "editors: SKIPPED vscode (need node, npm, and editors/vscode/node_modules)"
fi

nvim="${NVIM:-nvim}"
# mise.toml pins Neovim for the gate; a pinned Neovim that is not installed is an error, not a skip.
if ! command -v "$nvim" >/dev/null && grep -q 'neovim/neovim' "$root/mise.toml" && command -v mise >/dev/null; then
    nvim="$(cd "$root" && mise which nvim 2>/dev/null)" || { echo "editors: mise.toml pins neovim; run mise install" >&2; exit 1; }
fi
if command -v "$nvim" >/dev/null && "$nvim" --headless --clean -c 'if has("nvim-0.10") | qa! | else | cq | endif' >/dev/null 2>&1; then
    echo "editors: neovim"
    "$nvim" --headless --clean -l "$root/editors/neovim/test/smoke.lua" "$cpl" "$root"
else
    echo "editors: SKIPPED neovim (no nvim 0.10 or later; set \$NVIM)"
fi

vim_bin="${VIM_BIN:-vim}"
if command -v "$vim_bin" >/dev/null; then
    echo "editors: vim"
    VIM_BIN="$vim_bin" "$root/editors/neovim/test/smoke_vim.sh" "$cpl"
else
    echo "editors: SKIPPED vim (set \$VIM_BIN)"
fi

emacs="${EMACS:-emacs}"
if command -v "$emacs" >/dev/null; then
    echo "editors: emacs"
    CPL="$cpl" "$emacs" --batch -Q -l "$root/editors/emacs/smoke.el" 2>&1 | grep 'emacs smoke'
    # eglot runs `cpl lsp` as cp-mode names it; the shared request matrix goes to that command.
    python3 "$root/editors/lsp-smoke.py" emacs "$cpl" lsp
else
    echo "editors: SKIPPED emacs (set \$EMACS)"
fi

if command -v "${HX:-hx}" >/dev/null; then
    echo "editors: helix"
    "$root/editors/helix/smoke.sh" "$cpl"
else
    echo "editors: SKIPPED helix (set \$HX)"
fi

if rustup target list --installed 2>/dev/null | grep -qx wasm32-wasip1; then
    echo "editors: zed"
    "$root/editors/zed/smoke.sh" "$cpl"
else
    echo "editors: SKIPPED zed (rustup target add wasm32-wasip1)"
fi

ts="${TREE_SITTER:-tree-sitter}"
if command -v "$ts" >/dev/null; then
    echo "editors: tree-sitter"
    TREE_SITTER="$ts" "$root/editors/tree-sitter-cp/smoke.sh"
else
    echo "editors: SKIPPED tree-sitter (set \$TREE_SITTER)"
fi

if command -v java >/dev/null; then
    echo "editors: jetbrains"
    (cd "$root/editors/jetbrains" && ./gradlew -q buildPlugin test)
    echo "jetbrains build and tests: ok"
else
    echo "editors: SKIPPED jetbrains (no java)"
fi
