# Editors

Every editor talks to the same language server, `cpl lsp`: diagnostics as you
type, hover with types and doc comments, completion, signature help, go to
definition (into the stdlib too), references, rename, document symbols,
formatting, quick fixes and inlay hints. Put `cpl` on the `PATH`
(`cargo install --path crates/cpl-cli`) or point the editor at it.

| Editor | Where | Highlighting | Run, test, trace |
|---|---|---|---|
| VS Code | `editors/vscode/` | TextMate grammar | lenses, stress, trace view |
| Neovim and Vim | `editors/neovim/` | Vim syntax | Run, Trace and Stress lenses |
| Emacs | `editors/emacs/` | font-lock | `M-x compile` with `cpl run` |
| JetBrains IDEs | `editors/jetbrains/` | lexer, color settings | run configurations, gutter marker, trace tab |
| Helix | `editors/helix/` | none yet (needs a tree-sitter grammar) | `:sh cpl run %` |
| Zed | not yet | needs a tree-sitter grammar | |

`editors/smoke.sh` checks each setup against the real server (the keyword
lists against the compiler, then Neovim, Emacs and Helix when installed, and
the JetBrains plugin's build when `java` is);
`scripts/check.sh --full` runs it.

## Neovim (0.10 or later)

Add the folder to the runtime path; it sets the filetype, the highlighting and
the comment string, and starts `cpl lsp` for every `.cp` buffer with the
server's Run, Trace and Stress lenses (`<cmd>lua vim.lsp.codelens.run()<cr>`
runs the one under the cursor in a terminal split):

```lua
vim.opt.rtp:append('/path/to/cp-lang/editors/neovim')
vim.g.cpl_path = 'cpl' -- or the full path
```

With lazy.nvim, `{ dir = '/path/to/cp-lang/editors/neovim' }` does the same.
With nvim-lspconfig instead of the bundled autocommand, register the server
and keep the rest:

```lua
require('lspconfig.configs').cpl = {
    default_config = { cmd = { 'cpl', 'lsp' }, filetypes = { 'cp' }, root_dir = vim.fs.dirname },
}
require('lspconfig').cpl.setup({})
```

Vim gets the highlighting from the same folder (`set rtp+=...`); for the
server, use a client such as vim-lsp with `cmd: ['cpl', 'lsp']`.

## Emacs

```elisp
(add-to-list 'load-path "/path/to/cp-lang/editors/emacs")
(require 'cp-mode)
(add-hook 'cp-mode-hook #'eglot-ensure)   ; Emacs 29+, or #'lsp with lsp-mode
```

`cp-mode` registers `cpl lsp` with both eglot and lsp-mode; set
`cp-cpl-program` when `cpl` is not on the `PATH`. It claims `.cp` ahead of
Emacs's own entry, which calls `.cp` C++.

## JetBrains IDEs (2024.3 or later)

`cd editors/jetbrains && ./gradlew buildPlugin` writes
`build/distributions/cp-lang-jetbrains-<version>.zip`; install it with
Settings, Plugins, Install Plugin from Disk. The IntelliJ Platform's LSP API
ships only in the commercial IDEs (IntelliJ IDEA Ultimate, CLion, PyCharm
Professional and the rest), so the plugin needs one of them. Set the path to
`cpl` under Settings, Tools, CP Lang when it is not on the `PATH`. A `.cp`
file gets a run configuration on its first sample (`cpl run`, `cpl test` or
`cpl stress`), a run marker on `fn main`, and CP Lang: Show Trace
(Ctrl+Alt+T) opens `cpl run --trace --format html` in an editor tab.
`./gradlew test` runs the lexer tests and a smoke test in a headless IDE
against `target/debug/cpl` (or `$CPL`).

## Helix

Append `editors/helix/languages.toml` to `~/.config/helix/languages.toml`,
then `hx --health cp` shows the server. Helix highlights only through a
tree-sitter grammar, which CP Lang does not have yet.

## Zed

Zed adds a language only through an extension with a tree-sitter grammar, so
it waits on that grammar. Until then, open `.cp` files in another editor.
