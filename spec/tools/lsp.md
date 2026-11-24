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
- **Neovim and Vim** (the Milestone 1 editors): Neovim 0.12 or later gets
  diagnostics, completion, formatting and definition from one step,
  `cpl setup neovim`, with no checkout, plugin manager or lspconfig
  needed. Vim 8.2 or later gets highlighting, filetype, indent and `gq`
  through `cpl fmt` from `cpl setup vim`, and the server through the
  vim-lsp it installs beside them (or the vim-lsp or yegappan/lsp already
  installed); any other LSP client runs `cpl lsp`. (Rlsp.5, permits)
- **Running `cpl`**: the server says how an editor runs a lens's `cpl.exec`
  arguments or a command the user typed, so the editor only forwards and
  draws. `cpl/command` takes `{arguments, file?}` and answers `{argv, view}`:
  the arguments after `cpl` (none is `test`, `file` goes after the
  command), and `view` true when they ask for JSON whose `view` the editor
  draws (`cpl stress`, `cpl locate`) instead of a terminal. `cpl/commands`
  takes `{line}` and answers the `cpl` command names completing its second
  word. Neovim's plugin uses both (`crates/cpl-lsp/src/tests/command.rs`).
- **Import a Problem**: `workspace/executeCommand` of `cpl.import` with
  arguments `[path, port?]` waits on `127.0.0.1:port` (default `10043`)
  for Competitive Companion and answers `{port}`. Each problem it sends is
  created as `cpl new --listen` would create it in `path` (or the folder of
  the file `path` names, or the workspace's first folder when `path` is
  null), and the server opens it with
  `window/showDocument`; it reports waiting and failure with
  `window/showMessage`. JetBrains' Import a Problem sends only this
  command (`crates/cpl-lsp/tests/import.rs`).
- **Showing a run**: `workspace/executeCommand` runs `cpl.exec` itself: the
  arguments after `cpl`, with an object among them giving `stdin`,
  `expect` and flags by name (`{case: "i1"}` is `--case i1`); `stdin` may
  list choices, the first with text taken, and `typed: [line]` is the
  expected output typed with `\n` for line breaks (`[null]`, a cancelled
  prompt, runs nothing and answers `null`). It is completed by
  the client's `cp` settings (`cp.stress.args`, `cp.submit.copy`,
  `cp.liveValues.input`, `cp.run.timeoutMs`) and `--format json` (a trace's
  `html`, also sent as a `cpl/page` `{html}` notification). It answers
  `{view, html, teamcity, ok, code, stderr}` (`ok`: it exited 0;
  `teamcity`: a test run's `view.tests` as the service messages
  `cpl test --format teamcity` prints) when the run ends and
  shows the `view` through the protocol: the panel as a `window/logMessage`
  (Info), the status as a `window/showMessage`, each line's strongest mark
  and the status as inlay hints at the line's end (clipped to
  `cp.liveValues.maxLength`, the full text on hover), and its diagnostic
  among the document's, until the next edit. `cpl.show` `[uri, view]` shows
  a view another tool made, `cpl.clear` `[uri]` takes one away, and with
  `cp.liveValues.enabled` each save runs the samples quietly
  (`crates/cpl-lsp/src/tests/exec.rs`). With `initializationOptions.trusted`
  false (VS Code's Restricted Mode) `cpl.exec`, `cpl.import` and live values
  refuse to run, and `cp.inlayHints.enabled` false hides every inlay hint.
  JetBrains forwards its lenses' `cpl.exec` and draws only a trace's `html`
  and a test run's `teamcity` in IntelliJ's own test tree (`CpLsp4ijTest`).
- **Replay while debugging**: `initializationOptions.replayLenses`
  `"debugging"` shows the Replay lenses only after a `cpl/debugging`
  `{type: "cp"}` notification and until one with another `type` or none,
  asking `workspace/codeLens/refresh` at each
  (`crates/cpl-lsp/src/tests/lenses.rs`).

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
| Neovim and Vim | `g:cpl_path` | `"cpl"` | The `cpl` binary; `cpl setup` sets it to the one it ran as. |
| Neovim and Vim | `g:cpl_format_on_save` | `true` | Format through `cpl fmt` on every write. |
| Neovim and Vim | `g:cpl_autocomplete` | `true` | Complete as you type. |
| Emacs | `cp-cpl-program` | `"cpl"` | The `cpl` binary; `cpl setup emacs` sets it to the one it ran as. |
| JetBrains | `path` | `""` | The `cpl` binary (Settings, Tools, CP Lang); `cpl setup jetbrains` sets it to the one it ran as, and empty is `cpl` on the PATH. |
| Zed | `lsp.cpl.binary.path` | `"cpl"` | The `cpl` binary; unset is `cpl` on the PATH. |
| Zed | `lsp.cpl.binary.arguments` | `["lsp"]` | Arguments that start the language server. |

Evidence: `spec/conformance/tools-lsp/` drives `cpl lsp --stdio` for each
rule, and the LSP's own suites go deeper: `crates/cpl-lsp/tests/fixtures/` (diagnostics, completion,
navigation), `crates/cpl-lsp/src/tests/format.rs` and
`editors/neovim/test/smoke.lua`.
