# Errors checklist

| | |
|---|---|
| Kind | Checklist |
| Owner | diagnostics |
| Describes | every error, warning and runtime trap the toolchain shows; the reasoning behind them is in `handbook/guidelines/errors.md` |

Every rule an error, warning or trap must meet, one per heading. Each has a
stable id (`errors/R3`), one sentence, its reason, an example and what
enforces it. Tests, lints and the diagnostics quality register cite these
ids or headings instead of restating the rule. When a case is not covered,
decide it from `handbook/guidelines/errors.md`, then add the rule here.

# Reference

## Where it points

### R1. One error per root cause

Report the first thing the user got wrong, and nothing that follows from it.

- **Why:** the user fixes the first error and reruns; echoes bury the next real one.
- **Example:** `let x: Int = "a"` then `print(x + 1)` gets one E0302 at `"a"`, none at `x + 1`.
- **Enforced by:** `Sink` drops a second diagnostic inside a reported span and `Ty::Error` poisons further checks; `crates/cpl-cli/tests/recovery.rs` `recovery_scoreboard` (cascade rate).

### R2. The primary span is the token to edit

The primary span is in the user's file, on the token the user must change.

- **Why:** the user edits where the caret points.
- **Example:** for `rows.sort(by: (a, b) => (-b, a))` the caret is on `(a, b)`, not on `-b`.
- **Enforced by:** `recovery.rs` `recovery_scoreboard` (first error on the broken line); `crates/cpl-cli/tests/diagnostics.rs` `real_mistakes_name_the_place_the_rule_and_a_fix`.

### R3. Never a primary span inside std

An error that arises in a library has the user's call as its primary span and the library line in a note.

- **Why:** the user can't edit `std/`, and a std line tells them nothing about their own mistake.
- **Example:** a bad argument to `a.maximal(..)` points at the call in `main.cp`, with `std/seq.cp:140` as a note.
- **Enforced by:** `diagnostics.rs` `no_golden_error_points_into_std`; `crates/cpl-cli/tests/cli.rs` `an_error_inside_std_names_the_library_calls_that_led_there`.

### R4. A label marks the other half

When an error involves two places, the second carries a label.

- **Why:** "already declared" is useless without where.
- **Example:** E0405 on assigning a `let` labels the `let` line: "declared with `let` here".
- **Enforced by:** `crates/cpl-check/src/labels.rs` labels the declaration an error names when it is on another line; `crates/cpl-cli/tests/labels.rs` checks every golden and catalogue trigger.

## What it says

### R5. Name what the user wrote, in the spec's words

The message names the construct the user wrote and uses the spelling of `spec/GLOSSARY.md`, citing the rule id when the rule is not obvious.

- **Why:** the user knows their code, not the parser's state; one spelling per idea across error, docs and trace.
- **Example:** "`&&` is `and` in CP Lang", not "expected an expression".
- **Enforced by:** `crates/cpl-cli/tests/diagnostics.rs` goldens; review; `missing` as a vocabulary check over messages.

### R6. Types read as the user wrote them

Aliases, domains and views print by their names.

- **Why:** internal names are a second language the user never learned.
- **Example:** `Rev<List<Int>>` and `[V]Int`, not a lowered record or a monomorphized name.
- **Enforced by:** `diagnostics.rs` goldens.

### R7. Only suggest what exists

A did-you-mean, rename, import or help names a public item that runs, in the form it is called.

- **Why:** a suggestion the user can't use costs a second round trip.
- **Example:** `a.zip(b)` gets `zip(a, b)`; a method is suggested as a method.
- **Enforced by:** `crates/cpl-cli/tests/suggestions.rs` writes each did-you-mean in place, with its `use`, and checks the error is gone; `crates/cpl-cli/tests/catalogue.rs` `every_emitted_code_has_an_exact_trigger_and_a_clean_fix` applies each machine fix.

## The fix

### R8. Give a fix, or say what to do

If one edit is right, attach it with `Diagnostic::fix`; if several are plausible, attach each; if the right edit depends on intent, put the steps in `help` instead.

- **Why:** `cpl fix` and the editor can apply an attached fix in one keystroke.
- **Example:** `&&` gets the fix `and`; a two-parameter `sort(by:)` lambda gets help naming a key like `x => (x.name, x.score)`, not a guessed rewrite.
- **Enforced by:** `recovery.rs` `fix_it_scoreboard`; `crates/cpl-cli/tests/newcomer.rs` `newcomer_scoreboard`; `diagnostics.rs` `real_mistakes_name_the_place_the_rule_and_a_fix`.

### R9. A fix is safe to apply unread

An applied fix leaves a program that checks clean, prints what it printed before, and changes nothing when applied again.

- **Why:** fix-all applies fixes nobody reads, so a wrong one silently breaks working code.
- **Example:** removing `mut` from a parameter is not offered when a method call may change it.
- **Enforced by:** `crates/cpl-ir/src/fix.rs` applies a candidate only if the result re-checks; `crates/cpl-cli/tests/fix_behavior.rs`; `missing` for idempotence. Lint fixes follow the same rule (`handbook/guidelines/lints.md`).

## Warnings and traps

### R10. A warning is true

A warning fires only when it can show the problem; when the analysis can't see why code is safe, it stays silent.

- **Why:** one false warning teaches users to ignore all of them.
- **Example:** L0306 names an input inside the declared ranges that overflows; L0610 is silent on a `mut` parameter used through a method call.
- **Enforced by:** `crates/cpl-lint/src/corpus_tests.rs` `corpus_warnings_only_go_down`; each lint's `near_miss.cp`. The full lint rules are in `handbook/guidelines/lints.md`.

### R11. A trap names the values and the way out

A runtime trap shows the line, the values that broke the rule, the call stack, the test case number in multitest, and the CP Lang way to avoid it.

- **Why:** a trap happens on hidden input, where the user can't add a print.
- **Example:** R0202 on `a * b` shows both operands and suggests `I128(a) * b`.
- **Enforced by:** `crates/cpl-cli/tests/runtime_errors.rs` `every_runtime_code_traps_with_the_message_the_spec_shows` and `common_traps_say_the_cpl_fix`; `crates/cpl-cli/tests/trap_parity.rs`.

### R16. A warning about declared ranges shows its witness

A warning drawn from the declared ranges names an input those ranges allow that goes wrong, or the fact they already prove, and stays silent once the code tests it.

- **Why:** the user wrote the ranges from the statement, so a concrete input inside them is the one argument that convinces; a vague "may overflow" gets ignored.
- **Example:** L0306 on `a * b` with `a, b: 1..=10**10` says `a = 9223372037, b = 1000000000` overflows; L0307 on `if n >= 1:` with `n: 1..=10` says the range already proves it; L0305 on `a.max()!` stays quiet after `if a.len > 0:`.
- **Enforced by:** each bound lint's `before.cp` and `near_miss.cp` under `spec/lints/` (codes `L0302` to `L0308`), run by `cargo test -p cpl-lint`.

## Everywhere

### R12. Every tool shows the same diagnostic

`cpl check`, `cpl run`, `cpl build`, the editor and the playground report the same code, message, span and fix for a file, on every OS and with LF or CRLF line ends, by calling the same front end and `cpl_ir::fix`.

- **Why:** an error that shows in one tool and not another makes every tool suspect.
- **Example:** source.fixAll in the editor writes the same file as `cpl fix`.
- **Enforced by:** `crates/cpl-cli/tests/lsp_parity.rs`; `crates/cpl-wasm/tests/parity.rs` `fix_all_matches_cpl_fix`; editor CI jobs; `missing` for `check` vs `run` vs `build`.

## Codes

### R13. Every code is catalogued with a trigger, a fix and a page

A new code lands with its row in `spec/diagnostics.md`, a `REGISTRY` entry whose trigger emits exactly it and whose fix makes the trigger check clean, a `cpl explain` page, and for a lint a record (`lint.toml`, `explain.md`, `before.cp`, `after.cp`, `near_miss.cp`).

- **Why:** `cpl explain` and the docs are only as complete as the catalogue.
- **Example:** E0303 has a trigger, a fix and `cpl explain E0303`.
- **Enforced by:** `catalogue.rs` `every_defined_code_is_catalogued`, `every_emitted_code_has_an_exact_trigger_and_a_clean_fix`; `crates/cpl-lint/src/record_tests.rs`.

### R14. A code never changes meaning

A code keeps its meaning forever, and a retired code is never reused.

- **Why:** users search for codes, and old answers must stay right.
- **Example:** `E0306 position_arithmetic` stays listed as *(retired)* in `spec/diagnostics.md`, and new meanings get new codes.
- **Enforced by:** `crates/cpl-cli/tests/catalogue.rs` `codes_never_change_meaning`, which compares the catalogue with the last commit and fails on a deleted, renamed or un-retired code.

### R15. Every mistake a user hits becomes a case

An error a user, a blind reader or a bug report found unhelpful lands with its fix as a case in `crates/cpl-cli/tests/mistakes/`.

- **Why:** suites only know the mistakes someone wrote down.
- **Example:** `sort_by_takes_one_argument.cp`, from a contestant's first session.
- **Enforced by:** `diagnostics.rs` `real_mistakes_name_the_place_the_rule_and_a_fix`, which has no known-failure list.

## Before you land an error

- [ ] The primary span is on the token to edit, in the user's file (R2, R3).
- [ ] The message names what the user wrote and the rule, in the spec's words (R5).
- [ ] A right fix is attached, or `help` says what to do (R8, R9).
- [ ] Every name in the message and the fix exists and is public (R7).
- [ ] The error does not repeat for later uses of the same mistake (R1).
- [ ] A golden in `crates/cpl-cli/tests/diagnostics/` or a case in `crates/cpl-cli/tests/mistakes/` pins it, and `cargo test -p cpl-cli --test diagnostics --test catalogue` passes.
- [ ] `cargo test -p cpl-lsp parity` passes, so the editor shows the same (R12).

## Changes

- 2026-11-23: add R16, the rule the bound lints `L0302` to `L0308` enforce.
- 2026-11-23: first version, from the rules in `handbook/guidelines/errors.md` and the diagnostics register.
