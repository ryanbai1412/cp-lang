# Editors

Every editor talks to the same language server, `cpl lsp`: diagnostics as you
type, hover with types and doc comments, completion, signature help, go to
definition (into the stdlib too), references, rename, document symbols,
formatting, quick fixes and inlay hints. Put `cpl` on the `PATH`
(`cargo install --path crates/cpl-cli`) or point the editor at it.

| Editor | Where | Highlighting | Run, test, trace |
|---|---|---|---|
| VS Code | `editors/vscode/` | TextMate grammar | lenses, stress, trace view |
| Neovim and Vim | `editors/neovim/` | Vim syntax, or tree-sitter | Run, Trace and Stress lenses |
| Emacs | `editors/emacs/` | font-lock | `M-x compile` with `cpl run` |
| JetBrains IDEs | `editors/jetbrains/` | lexer, color settings | run configurations, gutter marker, trace tab |
| Helix | `editors/helix/` | tree-sitter (`editors/tree-sitter-cp/`) | `:sh cpl run %` |
| Zed | `editors/zed/` | tree-sitter (`editors/tree-sitter-cp/`) | `cpl run` in a terminal |

`editors/smoke.sh` checks the keyword lists against the compiler, then runs
each editor's smoke test when the editor is installed: each starts `cpl lsp`
the way that editor's setup does and checks `initialize` and the error it
publishes for a file (`editors/neovim/test/smoke.lua`, `editors/emacs/smoke.el`,
`editors/helix/smoke.sh`, and `editors/zed/smoke.sh` when Rust has the
`wasm32-wasip1` target). With the `tree-sitter` CLI it checks the grammar,
and with `java` it builds the JetBrains plugin. `scripts/check.sh --full` runs
it.

`editors/tree-sitter-cp/` is a tree-sitter grammar for highlighting: tokens,
`fn` names, calls, members, types and constants. Blocks and everything else
come from the server.

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

For tree-sitter highlighting, build the parser into the folder (or add
`editors/tree-sitter-cp` to nvim-treesitter's parser configs and `:TSInstall cp`);
the plugin starts it on every `.cp` buffer, with the queries in
`editors/neovim/queries/cp/`:

```sh
cd /path/to/cp-lang/editors && mkdir -p neovim/parser &&
  cc -O2 -shared -fPIC -I tree-sitter-cp/src tree-sitter-cp/src/parser.c -o neovim/parser/cp.so
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

Append `editors/helix/languages.toml` to `~/.config/helix/languages.toml`
with its grammar `path` pointing into your checkout, link the queries, and
build the grammar; `hx --health cp` then shows the server, the parser and the
highlight queries:

```sh
mkdir -p ~/.config/helix/runtime/queries
ln -s /path/to/cp-lang/editors/helix/runtime/queries/cp ~/.config/helix/runtime/queries/cp
hx --grammar build
```

## Zed

`editors/zed/` is a Zed extension: the language, highlighting from the
tree-sitter grammar, and `cpl lsp` as its server. In Zed, run
`zed: install dev extension` and pick `editors/zed` (it needs Rust with the
`wasm32-wasip1` target). It starts `cpl` from the PATH; to use another, set

```json
{ "lsp": { "cpl": { "binary": { "path": "/path/to/cpl", "arguments": ["lsp"] } } } }
```

Zed fetches the grammar at the commit `extension.toml` names; when
`editors/tree-sitter-cp` changes, point `rev` at the new commit.
