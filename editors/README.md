# Editors

Every editor talks to the same language server, `cpl lsp`: diagnostics as you
type, hover with types and doc comments, completion, signature help, go to
definition (into the stdlib too), references, rename, document symbols,
formatting, quick fixes and inlay hints. Put `cpl` on the `PATH`
(`cargo install --path crates/cpl-cli`) or point the editor at it.

| Editor | Where | Highlighting | Run, test, trace |
|---|---|---|---|
| VS Code | `editors/vscode/` | TextMate grammar | lenses, stress, trace view |
| Neovim and Vim | `editors/neovim/` | Vim syntax, or tree-sitter | Run, Trace, Test and Stress lenses; `:Cpl <command>` |
| Emacs | `editors/emacs/` | font-lock | `M-x compile` with `cpl run` |
| JetBrains IDEs | `editors/jetbrains/` | lexer, color settings | run configurations, gutter marker, trace tab |
| Helix | `editors/helix/` | tree-sitter (`editors/tree-sitter-cp/`) | `:sh cpl run %` |
| Zed | `editors/zed/` | tree-sitter (`editors/tree-sitter-cp/`) | `cpl run` in a terminal |

`editors/smoke.sh` checks the keyword lists against the compiler, then runs
each editor's smoke test when its toolchain is installed: each starts `cpl lsp`
the way that editor's setup does and checks `initialize` and the error it
publishes for a file (`editors/vscode/smoke.sh`, `editors/neovim/test/smoke.lua`,
`editors/neovim/test/smoke_vim.sh`, `editors/emacs/smoke.el`,
`editors/helix/smoke.sh`, and `editors/zed/smoke.sh` when Rust has the
`wasm32-wasip1` target). With the `tree-sitter` CLI it checks the grammar,
and with `java` it builds and tests the JetBrains plugin. `scripts/check.sh --full`
runs it.

`editors/tree-sitter-cp/` is a tree-sitter grammar for highlighting: tokens,
`fn` names, calls, members, types and constants. Blocks and everything else
come from the server.

## VS Code

One step, once `cpl` and VS Code's `code` command are installed:

```sh
cpl setup vscode
```

It downloads the edge release's `cp-lang.vsix` and runs `code --install-extension`
(`--vsix FILE` installs a file you have; `CPL_CODE=codium` or `cursor` picks
another editor). Open a `.cp` file: highlighting, diagnostics, completion,
go to definition, format on save, and the Run, Test, Stress and Submit commands.
`CP: Import a Problem from Competitive Companion` waits for the browser
extension (`cpl new --listen`) and creates `a.cp` with its samples beside the
open file; `CP: Add a Test from the Selection or Clipboard` saves the next case
(`cpl test --add -`) and runs it.

Or build it from this repo (no marketplace needed):

```sh
cargo install --path crates/cpl-cli
cd editors/vscode
npm ci
npm run package
code --install-extension cp-lang.vsix
```

Or run an Extension Development Host without packaging:

```sh
cd editors/vscode && npm ci && npm run compile
code --extensionDevelopmentPath="$PWD" /path/to/your/workspace
```

`editors/vscode/smoke.sh` runs the grammar and unit test suite (`npm test`)
and checks the live `cpl lsp` handshake. See `editors/vscode/README.md` for
settings, snippets, and extension-host integration tests.

## Neovim (0.10 or later)

One step, once `cpl` is installed:

```sh
cpl setup neovim
```

Open a `.cp` file: highlighting, diagnostics, completion (as you type `.`, and
Ctrl-X Ctrl-O any time; Neovim 0.11 or later for as-you-type), `gd` to the
definition, `K` for hover, formatting on every write, the Run, Trace, Test and
Stress lenses (`<cmd>lua vim.lsp.codelens.run()<cr>` runs the one under the
cursor) and `:Cpl <command>` (bare `:Cpl` runs `cpl test` on the file). It
writes the plugin to `stdpath('data')/site/pack/cpl/start/cplang`; run it again
after `cpl self update`. With [nvim-dap](https://github.com/mfussenegger/nvim-dap)
installed, `:DapContinue` on a `.cp` file debugs it through `cpl dap` with no
setup: breakpoints, stepping, and time travel back with `require('dap').step_back()`
and `require('dap').reverse_continue()`. Settings, all optional:

```lua
vim.g.cpl_path = 'cpl'             -- when cpl is not on the PATH
vim.g.cpl_format_on_save = false   -- keep your own formatting
vim.g.cpl_autocomplete = false     -- keep your completion plugin's menu
```

From a checkout instead, add the folder to the runtime path
(`vim.opt.rtp:append('/path/to/cp-lang/editors/neovim')`, or with lazy.nvim
`{ dir = '/path/to/cp-lang/editors/neovim' }`). With nvim-lspconfig instead of
the bundled autocommand, register the server and keep the rest:

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

Vim: `cpl setup vim` installs the filetype and highlighting into
`~/.vim/pack/cpl/start/cplang` (`~/vimfiles` on Windows), and registers
`cpl lsp` with vim-lsp or yegappan/lsp when one of them is installed. `gq` formats
through `cpl fmt --stdin`. To format on write:

```vim
autocmd BufWritePre *.cp let b:cpl_view = winsaveview() | silent execute 'keepjumps normal! gggqG' | call winrestview(b:cpl_view)
```

## Vim (8.2 or later)

Add the Neovim folder to the runtime path for highlighting, filetype, indent,
`gq` through `cpl fmt`, and `cpl lsp` through vim-lsp or yegappan/lsp when
either is installed:

```vim
set rtp+=/path/to/cp-lang/editors/neovim
filetype plugin indent on
syntax on
```

The wrong-answer loop is two shell commands: `:!cpl stress %` finds and saves
a small failing case in `<file>.tests/`, and `:!cpl test %` reruns every saved
case after the fix.

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

One step, once `cpl` is installed and the IDE has been opened once:

```sh
cpl setup jetbrains
```

It unpacks the edge release's plugin into every installed IDE's plugins folder
(`--zip FILE` installs a file you have, `--dir DIR` one plugins folder); restart
the IDE. Or build it: `cd editors/jetbrains && ./gradlew buildPlugin` writes
`build/distributions/cp-lang-jetbrains-<version>.zip`; install it with
Settings, Plugins, Install Plugin from Disk. The commercial IDEs (IntelliJ IDEA
Ultimate, CLion, PyCharm Professional and the rest) run `cpl lsp` through the
platform's own LSP client; IntelliJ IDEA Community and PyCharm Community lack
it, so there the plugin runs it through the LSP4IJ plugin, which `cpl setup
jetbrains` installs beside it. `cpl setup jetbrains --check`, after a restart,
reads the IDE's log and fails unless CP Lang loaded. `./gradlew verifyPlugin`
checks the plugin against IDEA Community and Ultimate. Set the path to
`cpl` under Settings, Tools, CP Lang when it is not on the `PATH`. A `.cp`
file gets a run configuration on its first sample (`cpl run`, `cpl test` or
`cpl stress`), a run marker on `fn main`, and CP Lang: Show Trace
(Ctrl+Alt+T) opens `cpl run --trace --format html` in the CP Lang Trace tool
window. The file marks the line of the step the trace shows, and a click on a
line in the trace moves the caret there.
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
