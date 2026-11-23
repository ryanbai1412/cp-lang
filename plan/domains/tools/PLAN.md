# tools plan

The detail behind the [README](README.md): workstreams and their order, why each milestone, alternatives considered, risks, what comes first.

## Milestone 1 order (from 2026-11-22)
[plan/product/MILESTONE-1.md](../../product/MILESTONE-1.md) sets the order; the workstreams below serve it and anything not on this path waits.
1. **Run is the sample runner.** `cpl run a` with no input and a terminal on stdin runs `a.tests/` like `cpl test`; a number or file name picks one case; piped input runs once as today. Why first: Ryan's experience doc makes it the default loop (J6), and today `cpl run a` waits on stdin.
2. **Short forms.** `cpl b` for `build`, `-c` for `--copy`; generalise to a small alias table in the command table, not one-off cases.
3. **Custom tests.** `cpl test a --add [FILE|-]` writes the next `a.tests/<n>.in` (and `.out` with `--expect`); VS Code `CP: Add Test`.
4. **Companion from the editor.** VS Code `CP: Listen for Competitive Companion` runs `cpl new --listen` and opens the new file.
5. **VS Code install and polish.** Release `.vsix` (with infra), format on save default for `.cp`, a cold-install probe of steps 3-6, every stumble fixed.
6. **Contest-flow golden test.** One test drives new → run → test --add → stress → b -c on a Companion fixture, so the scenario cannot regress.

Deferred: workstream 3 (explain), the rest of 2 (shared parser beyond new flags), and tier-2 editor polish.

## Workstreams
1. **Land in-flight work.** `devin/wip-editor-lsp` (5 commits, based on `0fc84cf7`): edition-aware `use std.` completion, a v0.3 LSP feature test, a corpus latency benchmark, VS Code and Vim smoke tests, VS Code install docs, the synced VS Code prelude/stdlib tables (fixes the 1 failing VS Code unit test). Rebase, gate, land; drop anything `git cherry` shows superseded.
2. **Command contract.** One table of commands (name, one-line summary, flags, whether it prints results, owner lane) that drives `cpl --help`, each `<cmd> --help`, dispatch, and a `cpl --help --json` the site reads instead of scraping text. A fast-gate test diffs the table against `spec/tools/cli.md`. Golden tests per command: help, one success, one usage error, exit code. A shared `Args`/`Output` helper (positional, `--flag value`, `--json`, usage error → exit 3) that crates adopt one command at a time. Split `crates/cpl-cli/src/lib.rs` by command (`check.rs`, `build.rs`, `explain.rs`, `run.rs`) along the way, under the 2,000-line cap.
3. **Explain and hover from one source.** A consumer contract test in `cpl-lsp`/`cpl-cli` stating what the representation query must return (type, size, niche, storage, per-array bytes, peak memory, specialization chosen, checks left), filed with compiler and perf. Until the query lands, `cpl explain` renders the sections the shared core already exposes (closure lowering, dropped fields, hover types) and says which are missing instead of inventing numbers. Then `explain --json` and hover both render the same records.
4. **Editors.** Get the 5,000-line latency green (incremental re-check on open/edit, reuse of `cpl_check::Database` between requests), then enforce `latency.rs`'s 5,000-line budget in the gate (it runs there in debug, where only 500 lines is enforced) with a margin for the 2-core gate machine. Tier-1 smoke tests (VS Code, Neovim) in the gate; tier 2 run when installed. VS Code `.vsix` per release (with infra); JetBrains built in CI.
5. **Contest tooling gaps.** `fetch` (samples from a URL into `d.tests/`), `locate` (with debugger/stress), `migrate` (with stdlib pins), per Q tools-1's answer.

Order: 1 → 2 → (3 ∥ 4) → 5. 1 first because it is done work rotting against a moving main. 2 before 3-5 because every later command lands on the shared table and helpers instead of adding another hand-rolled parser.

## Why each milestone
- **M1 Honest contract**: a contestant's first contact is `cpl --help` and the site's CLI page; today the spec names 7 commands that don't exist and omits 7 that do. Nothing else in this lane is checkable until the contract is a test.
- **M2 Explain is the truth**: goal 3 ("you can see and choose why") and the design rule "nothing secret under the hood ... `cpl explain` shows it" rest on this command, and it currently shows closure lowering only.
- **M3 Editors shipped**: goal 4's "live feedback in the editor" and the 100 ms budget; the test that guards it is red at 5,000 lines in release, and the gate only enforces 500.

## Alternatives considered
- **A CLI framework (clap) instead of a hand-rolled table.** Rejected for now: adds the workspace's first heavy dependency, changes every help text at once, and `cpl-wasm` wants a small binary. Reverse: the table is data, so a later switch only replaces the parser.
- **Spec follows the binary wholesale.** Rejected: `fetch`, `locate`, `migrate` are real contest needs. **Binary follows the spec wholesale**: rejected: `check`, `fix`, `self` are used daily and `trace`/`debug` are better as `run --trace` and the editor's debugger. Hence Q tools-1's mixed answer.
- **`cli.json` emitted by the binary vs scraped by `gen-tools.mjs`.** Emitting it from the table; the scraper stays as the fallback until the site switches (a site-lane change, requested in its worklog when ready).
- **Explain computing its own layouts in `cpl-cli`.** Rejected: duplicates codegen's decisions and drifts; the shared-core rule says ask for the query.
- **LSP rewritten on a salsa-style incremental engine.** Deferred: the compiler lane owns incrementality; first measure where the 161 ms goes (parse vs check vs lint proofs for hints).

## Risks
- **Corpus is half red on v0.3** (450 of 877 ports check, `measure.py corpus`): explain/LSP corpus tests can only use the clean half until compiler/stdlib catch up. Mitigation: tests take "every port that checks", so coverage grows on its own.
- **Representation query not prioritised by compiler/perf** → M2 stalls. Mitigation: the contract test and the honest "not yet shown" sections; escalate in the compiler worklog, not a workaround.
- **Latency numbers are noisy on a 2-core VM** (two runs: open-to-diagnostics 132 ms then 161 ms). Mitigation: gate on p90 with a margin and a fixed corpus file; track the median as a metric.
- **Editor tests need binaries the VM lacks** (nvim ≥ 0.10, emacs, hx, tree-sitter) and Gradle downloads are blocked or rate-limited. Mitigation: ask infra to add nvim to the environment; JetBrains stays CI-only.
- **Touching `lib.rs` collides with other lanes** (debugger code and tests live there). Mitigation: split by command in small commits, land each fast, move debugger files only with the debugger lane.

## Next
Milestone 1 order, items 1-3, each a small commit with its golden test.
