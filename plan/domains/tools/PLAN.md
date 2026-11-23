# tools plan

The detail behind the [README](README.md): workstreams and their order, why each milestone, alternatives considered, risks, what comes first.

## Milestone 1 order (from 2026-11-22, deadline 09:30 PST 2026-11-23)
[plan/product/MILESTONE-1.md](../../product/MILESTONE-1.md) and Ryan's 21:30 update set the order; anything not on this path waits. Checkpoint in worklog.md every 2 hours.
1. **Neovim LSP** (done: `cpl setup neovim`). The friend uses Neovim/Vim. `editors/neovim/test/smoke.lua` gains completion, formatting and go-to-definition; `editors/README.md` gets one copy-paste setup; CI runs the smoke test headless (nvim is pinned in mise.toml). Vim's setup checked by `smoke_vim.sh`.
2. **Windows CLI.** CRLF in inputs and `.out` files, Windows paths in file resolution and messages, `clip.exe` for `-c`; tests that run on the Windows runner.
3. **Contest-flow golden test** on all three platforms: new (Companion fixture) → run → test --add → stress → b -c.
4. **Done already:** `cpl run` runs the samples (440aa2111), `cpl b -c` and short forms (dc8a5c361), `cpl test --add` (b6394311a).
5. **VS Code**: release `.vsix`, format on save, go-to-definition, Companion listen and Add Test commands; smoke in CI.
6. **JetBrains** (Ryan, 21:34: "i care about neovim, vscode, and jetbrains"): the plugin over `cpl lsp` with highlighting, diagnostics, completion, formatting and definition; one-step install from the release zip; a CI smoke build and test (CI reaches Gradle; this VM can't).

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

## General fixes owed
Each local fix tonight, and the mechanism that would remove its whole class.

| Instance (local fix) | General fix | Owner |
|---|---|---|
| `cpl test cf2272A` read the g++ binary `cf2272A` beside `cf2272A.cp` as source (BLOCKERS 41) | done: one resolver for every command that takes a file or problem; a bare name that is missing or not text is its `.cp` | tools |
| `cpl listen` (BLOCKERS 14) was "unknown command" though `cpl new --listen` exists | done: every unknown command names the one meant, from other tools' spellings or within two edits; one `cpl_diag::edit_distance` replaces cpl-check's and cpl-lint's copies | tools |
| VS Code grammar scoped `None` as a type after `none` became `None` (835de99a1), and the extension said 0.2.2 beside a 0.3.0 release | done: the grammar's literal words come from `cpl_syntax::token::LITERAL_WORDS` (tested to parse as literals) through `language-data.js`; the .vsix and plugin versions come from the workspace `Cargo.toml`. tree-sitter-cp now parses `None` (and the old `none`), and `editors/smoke.sh` fails when its literal words differ from `LITERAL_WORDS` | tools |
| `build-wasm.sh` ran `wasm-opt` when installed: binaryen 132 rejected rustc's bulk-memory output, and when it ran the file gzipped larger | done: no optional size step, so the bytes match on every machine | tools |
| `cpl.wasm` is 1,912,009 B gz; std text is 166 KB of it | std modules past the prelude fetched by the playground when a program `use`s them (a module-provider hook in `cpl-check`, as `CPL_STD_DIR` is natively) | site + tools |
| The interpreter is built twice in wasm (traced and untraced, 395 KB raw) | one interpreter with a dynamic observer in the wasm build, if debug-mode speed allows | debugger |
| Windows CI: editor tests split CRLF repo files (normalized in the test) | `.gitattributes` `* text=auto eol=lf`, with `-text` on the CRLF test fixtures, so every checkout is LF | infra |
| Built C++ prints `\r\n` on Windows, the interpreter `\n` (cpl-cli tests normalize) | Done: binary stdout under `_WIN32` in runtime.rs's `out` piece; the profiler now inserts its tally before the first namespace, not after the last `#include` | tools (perf reviews) |
| VS Code test compared `diagnostic.code` to a string; the server's new `codeDescription` link made it an object | an LSP-level diagnostics test in cpl-lsp already pins `codeDescription`; editor tests should assert on the protocol fields they need, not VS Code's shape of them | tools |
| `--checked` linked asan/ubsan, which MinGW lacks | Done: fall back to trapping UBSan when the link names a missing sanitizer runtime | tools |
| The LSP spells Windows paths `c:/...`; VS Code's memory-lens test compares via `Uri.file` | done: `uri::native_path` spells every lens argument the OS's way | tools |
| `listen_takes_a_problem_posted_by_the_extension` released its port-0 socket and bound the port again, racing other tests | done: `cpl_contest::new::listen_on` takes a bound socket, so a test holds its port from bind to accept | tools |
| JetBrains `CplTest` compared `File.path` (`\` on Windows) to `/` paths | editor tests compare paths in one spelling (`invariantSeparatorsPath`, `Uri.file`); the Windows editor jobs catch the rest | tools |
| The JetBrains plugin required `com.intellij.modules.ultimate`, so Community refused it and `cpl setup` still said done | done: optional dependencies per LSP client (decision 367), `verifyPlugin` against Community and Ultimate in CI, and `cpl setup <editor> --check`/`code --list-extensions` confirm the install | tools |
| Neovim and JetBrains find the first wrong call by reading `cpl stress`/`cpl locate`'s text (`solution ... at line N`) | `cpl lsp` publishes the last stress or locate divergence as a diagnostic on its line (from the `--json` `divergence` both commands print), so every LSP editor shows it with no output parsing | tools |
| cpl-contest tests drive `#!` shell scripts, so they're `cfg(unix)`; cpl-cli's `stress_judges_with_the_interactor_in_the_file` failed on Windows (`%1 is not a valid Win32 application`) once its step ran under bash | done for scripts: every program `Program::resolve` makes (`cpl stress`, `cpl locate`, the CLI's) runs a script through its `#!` interpreter on every OS (the path, else its name on the PATH: Git for Windows' `sh`), with a clear error when there is none; the cpl-contest tests can drop `cfg(unix)` where Windows CI has `sh` | a portable test-program helper (tiny `.cp` programs run by `cpl`) so they run on Windows too | tools |
| `cpl migrate` printed `.\a.cp` on Windows; `gen.sh` got `\\?\D:\...`, which Git for Windows' `sh` can't open (exit 127) | done: every path `cpl` prints goes through `cpl_diag::shown_path` / `Path::shown()` (`/`, no `\\?\`); golden tests compare with LF line ends; only argv and env paths keep the OS spelling | tools |
| `peak_memory` reads `/proc`: no MB and no MLE on macOS and Windows (bench test checks MB on Linux only) | per-platform peak memory (`proc_pid_rusage`, `GetProcessMemoryInfo`) in cpl-stress | debugger |
| Hand-kept VS Code tables missed `IndexOp` (and earlier `dot`, `cross`) | done: `editors/vscode/scripts/language-data.js` rewrites both from prelude.rs and std/ on every `npm run compile` and `bundle` | tools |
| Stale `d.cp` fixture (`.after` as a cut) broke the VS Code integration | `cpl check` every editor fixture in the gate | tools |
| clippy's `items_after_test_module` in cpl-lsp reached main | the land step runs `cargo clippy --all-targets -D warnings` on touched crates | infra |
| `@vscode/test-electron` 2.5 couldn't launch VS Code 1.147 on macOS (moved to 3.1) | dependency updates for `editors/vscode` on a schedule | tools |
| VS Code ran a stale `target/release/cpl` over a fresh debug build | done: `resolveServer` takes the newer workspace build | tools |
| Edge release stays at an old commit while any CI job is red, so `cpl setup` isn't installable | publish edge from the newest commit whose required jobs pass, not only fully green runs | infra |
| Headless Neovim on Windows draws no terminal output; the smoke runs the lens's command itself there | smoke tests check commands' output directly and the editor's terminal only where it renders | tools |
| Windows Neovim: `:Cpl check` and the run lens draw nothing in a headless terminal buffer | done: every terminal check in smoke.lua goes through `terminal_shows`, which runs the command itself on Windows | tools |
| VS Code: a debug session one integration test left stopped added Replay lenses to the lens test | done: the runner stops debugging after every test | tools |
| LSP range and on-type formatting format the whole file and send back only the changed lines that touch the selection; a file that does not parse gets no edits, so on-type formatting is silent mid-edit | `cpl-fmt` formats each top-level item that parses and leaves the rest as written | lint-fmt |
| `jetbrains_names_match_the_gradle_build` compared a `gradle.properties` line with `\n`; a Windows checkout has CRLF, and CI's Windows cli step hid the failure because PowerShell reports only its last command | done: the test reads by line, and every multi-line Windows step runs under `shell: bash` (`-e`), so any failing command fails it | tools |
| `SegTree` completion and hover listed internal `m`, `t`, `lz`: editors now show only `pub` methods and documented fields of another module's type (decision 351) | visibility in the language: fields and methods private to their module unless `pub`, enforced by the checker, so editors use the checker's rule (QUESTIONS) | spec + compiler |

## Next
Milestone 1 order, items 1-3, each a small commit with its golden test.
