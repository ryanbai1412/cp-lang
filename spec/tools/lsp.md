# Language server and editors

Status: draft for 0.3.0. Sources: `plan/60-editor.md`, decisions 29,
149. One compiler serves `cpl` and the LSP.

- **Diagnostics** as you type, cheapest first (syntax, names, types,
  domains, borrows, ranges); one error per root cause; each with its
  code, an index link and fixes as code actions. Cost and overflow
  warnings show the estimate on hover. (Rlsp.1)
- **Hover**: the type, where `None` lives, storage width and peak bytes,
  proven ranges, the cost of a call, and representation choices
  (niche, width from a range, specialization chosen by a law). (Rlsp.6)
- **Storage sizes** (decision 149): hover on a type or variable gives its
  size, field storage order, padding and niche; inlay hints after array
  and structure declarations give element size and peak bytes at the
  input bounds (`var dist: [V]?Int  // 8 B x 2e5 = 1.6 MB`); a code lens
  above the program shows total peak memory against the judge profile;
  padding over 20 % and `wide_array` are quick fixes. The numbers come
  from the same computation as `cpl explain`. (Rlsp.7)
- **Inlay hints**: inferred types, parameter names past two positional
  arguments, opt-in live sample values per top-level `let`. (Rlsp.8)
- **Completion** ranked by type (positions of `V` inside `a[` for
  `a: [V]T`, operation values where an `Op<T>` is expected, unimported
  stdlib names with the `use` edit); signature help with bound
  explanations. Completion offers only what the checker accepts at the
  cursor: each item comes from the checker's own resolution of the
  receiver and scope (a method whose `self` type and bounds the receiver
  meets, a name in scope or importable), so accepting one never adds an
  error. There is no hand-kept completion table; `.rev` on a generator is
  not offered because the checker rejects it (decision 513). (Rlsp.2)
- **Navigation and refactoring**: definition, references, rename,
  extract nested fn, inline, add dimension to a DP table, convert array
  to map. Go to definition reaches stdlib sources too. (Rlsp.3)
- **Editors**: a VS Code extension (highlighting, run/test/stress
  buttons, debugger, trace panel, judge in the status bar); any LSP
  client works for the rest. Each editor's settings are exactly the ones
  in the table below, at most 15 per editor, each with the default its
  plugin uses and listed in that editor's README. (Rlsp.9)
- **Formatting**: `textDocument/formatting` returns exactly what `cpl fmt`
  writes, so an editor's format-on-save and the CLI agree. (Rlsp.4)
- **Neovim and Vim** (the Milestone 1 editors): Neovim 0.10 or later gets
  diagnostics, completion, formatting and definition from one step,
  `cpl setup neovim`, with no checkout, plugin manager or lspconfig
  needed. Vim 8.2 or later gets highlighting, filetype, indent and `gq`
  through `cpl fmt` from `cpl setup vim`, and the server through the
  vim-lsp it installs beside them (or the vim-lsp or yegappan/lsp already
  installed); any other LSP client runs `cpl lsp`. (Rlsp.5, permits)

Editor settings, every one each editor exposes
(`crates/cpl-cli/tests/editor_settings.rs` checks this table against each
plugin). Helix has none: its `languages.toml` is the user's own config.

| Editor | Setting | Default | What it does |
|---|---|---|---|
| VS Code | `cp.server.path` | `"cpl"` | The `cpl` binary: the PATH, then the workspace's `target/`. |
| VS Code | `cp.server.args` | `["lsp"]` | Arguments that start the language server. |
| VS Code | `cp.trace.server` | `"off"` | Log the LSP messages. |
| VS Code | `cp.codeLens.enabled` | `true` | Run, Stress and Trace lenses (a Trace lens above each fn but `main`). |
| VS Code | `cp.inlayHints.enabled` | `true` | Inlay hints: types, proven ranges, domain loops. |
| VS Code | `cp.liveValues.enabled` | `false` | Each top-level `let`'s value after a save. |
| VS Code | `cp.liveValues.input` | `""` | The sample live values run on; empty is the first. |
| VS Code | `cp.liveValues.maxLength` | `60` | Longest inline value. |
| VS Code | `cp.submit.copy` | `true` | Copy the C++ when Submit finds it ready. |
| VS Code | `cp.stress.args` | `[]` | Extra `cpl stress` flags. |
| VS Code | `cp.run.timeoutMs` | `10000` | Kill a run or stress run after this long. |
| VS Code | `cp.companion.port` | `10043` | The port Import a Problem waits on. |
| Neovim and Vim | `g:cpl_path` | `""` | The `cpl` binary; empty is the PATH, then `~/.cpl/bin`, then `~/.cargo/bin`. |
| Neovim and Vim | `g:cpl_format_on_save` | `true` | Format through `cpl fmt` on every write. |
| Neovim and Vim | `g:cpl_autocomplete` | `true` | Complete as you type. |
| Emacs | `cp-cpl-program` | `"cpl"` | The `cpl` binary. |
| JetBrains | `path` | `""` | The `cpl` binary (Settings, Tools, CP Lang); empty is `$CPL`, the PATH, then `~/.cpl/bin`. |
| Zed | `lsp.cpl.binary.path` | `"cpl"` | The `cpl` binary; unset is `cpl` on the PATH. |
| Zed | `lsp.cpl.binary.arguments` | `["lsp"]` | Arguments that start the language server. |

Evidence: `spec/conformance/tools-lsp/` drives `cpl lsp --stdio` for each
rule, and the LSP's own suites go deeper: `crates/cpl-lsp/tests/fixtures/` (diagnostics, completion,
navigation), `crates/cpl-lsp/src/tests/format.rs` and
`editors/neovim/test/smoke.lua`.
