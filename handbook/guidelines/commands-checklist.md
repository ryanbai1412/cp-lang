# Commands and editor features: checklist

| | |
|---|---|
| Kind | Checklist |
| Owner | tools |
| Applies to | Every `cpl` command, flag, editor feature and playground call |

Every rule for commands and editor features, one per heading, each with
its identifier (`commands/R1`), the rule in one sentence, its reason, an
example, and what enforces it. The ideas behind them are in
`handbook/guidelines/commands.md`. Tests cite a rule by its heading, for
example `handbook/guidelines/commands-checklist.md#r1-help`.

# Reference

## Commands

### R1. Help

`commands/R1`: every command has `--help` listing its options, and every
help line fits in 80 columns.
- Reason: contestants learn the tool from `cpl --help`, in any terminal.
- Example: `cpl test --help` names `--checker`, `--add` and `--json`.
- Enforced by: `crates/cpl-cli/tests/golden.rs`
  (`every_command_has_help_and_rejects_an_unknown_option`,
  `every_help_fits_80_columns`).

### R2. Usage errors

`commands/R2`: an unknown option or a missing argument is a usage error
that shows the command's usage line.
- Reason: the fix is in the message, not in a search.
- Example: `cpl test --chekcer c.py` says the option is unknown and
  prints `usage: cpl test [FILE|FOLDER] ...`.
- Enforced by: `crates/cpl-cli/tests/golden.rs`
  (`every_command_has_help_and_rejects_an_unknown_option`).

### R3. One spelling

`commands/R3`: one flag per idea, spelled the same on every command.
- Reason: a flag learned on one command works on the next.
- Example: `--json` everywhere, with `--format json` as the same flag.
- Enforced by: `crates/cpl-cli/tests/golden.rs`
  (`json_and_format_json_are_one_flag_on_every_command_with_json`).

### R4. The spec names the commands

`commands/R4`: `spec/tools/cli.md` and the binary list the same commands
and flags.
- Reason: the spec is normative; a flag only one side knows is a bug.
- Example: adding `cpl test --add` adds it to `spec/tools/cli.md`.
- Enforced by: `crates/cpl-cli/tests/spec_sync.rs`, whose allowlist only
  shrinks.

### R5. Exit codes

`commands/R5`: the same outcome exits with the same code on every
command: 0 success, 1 errors in the source, 2 usage, 3 the program or a
test failed.
- Reason: editors, scripts and `cpl watch` act on the code.
- Example: `cpl test` with a wrong answer exits 3, as `cpl run` does on a
  trap.
- Enforced by: `crates/cpl-cli/tests/golden.rs`.

### R6. Machine output

`commands/R6`: `--json` output has stable field names in a fixed order,
one object per line for a stream, and is valid JSON even on failure.
- Reason: editors parse it, and a reordered field breaks them silently.
- Example: `cpl new --json` prints `{"path": ...}` per created file.
- Enforced by: `crates/cpl-json` (objects are ordered maps),
  `crates/cpl-cli/tests/golden.rs`.

### R7. Cold start

`commands/R7`: every command starts within the cold-start budget in
`plan/domains/tools/metrics.toml`, on the largest corpus solutions too.
- Reason: `cpl t` runs dozens of times a contest.
- Example: `cpl check` on the largest port.
- Enforced by: `crates/cpl-cli/tests/cold_start.rs`.

### R8. Paths

`commands/R8`: every path a command prints goes through
`cpl_diag::Shown`, so it opens on the user's OS.
- Reason: a path that does not open costs a search.
- Example: `a.cp:3:5` relative to the working folder, with `\` on Windows.
- Enforced by: `crates/cpl-cli/tests/golden.rs`.

### R9. Commands are functions

`commands/R9`: a command is `fn(args, stdout, stderr) -> u8` and never
exits the process.
- Reason: the playground and the tests call the same function as the
  terminal.
- Example: `cpl_cli::run(&["check", "a.cp"], &mut out, &mut err)`.
- Enforced by: `crates/cpl-wasm/tests/parity.rs`.

## Editor features

### R10. One entry point per job

`commands/R10`: each job has one function that the CLI, the language
server, the playground and the editor UIs call; a second caller lands with
a parity test.
- Reason: two implementations of one job drift apart.
- Example: `cpl fix`, fix-all and the playground call
  `cpl_lsp::fix::document`.
- Enforced by: `crates/cpl-cli/tests/lsp_parity.rs`,
  `crates/cpl-wasm/tests/parity.rs`.

### R11. Ask the analysis

`commands/R11`: a tool reads types, ranges, names and costs from the
checker's analysis and never re-derives them.
- Reason: a guess is right on examples and wrong on real files.
- Example: hover's storage line is the checker's representation query.
- Enforced by: the `own_databases` and `consumer_rederived` ratchets in
  `scripts/debt-ratchet.py`.

### R12. Half-typed code

`commands/R12`: every request answers on any text and never reports an
internal error.
- Reason: the contestant is always mid-edit.
- Example: completion after `rows.sort(by: (a, b) =>` with no body yet.
- Enforced by: `crates/cpl-lsp/tests/fuzz.rs`.

### R13. Latency

`commands/R13`: every request answers within the budget in
`plan/domains/tools/metrics.toml` at 5,000 lines, on each OS.
- Reason: a late completion is a missing one.
- Example: hover p95 on a generated 5,000-line file.
- Enforced by: `crates/cpl-lsp/tests/latency.rs`.

### R14. Measured over the corpus

`commands/R14`: completion, hover and go to definition are measured by
replaying every corpus solution, each measure a ratchet that only goes up.
- Reason: fixtures test what we thought of; the corpus is what
  contestants type.
- Example: retyping `print(n)` must offer `n` once `n` is typed.
- Enforced by: `crates/cpl-lsp/tests/completion_replay.rs`.

### R15. Same cases, same order

`commands/R15`: editors find, order and judge a problem's cases as
`cpl test` does, by calling it or by a test against `cpl-contest`.
- Reason: a test panel that lists other cases than the terminal is not
  trusted.
- Example: numbered cases by number, then the rest by name.
- Enforced by: `editors/vscode/test/unit/results.test.ts`,
  `editors/jetbrains/src/test/kotlin/dev/cplang/ide/CplTest.kt`.

## Changes

- 2026-11-23: first version, folding the rules stated in the tools
  register, the CLI goldens and the editor design.
