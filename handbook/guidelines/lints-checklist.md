# Lints checklist

| | |
|---|---|
| Kind | Checklist |
| Owner | lint-fmt |
| Explained in | `handbook/guidelines/lints.md` |

Every rule a lint, its record and its fix must follow, one per heading.
Cite a rule by its identifier, such as `lints/R5`, or by its heading
anchor. The rules a lint enforces on user code are not here: those live in
the checklist for that kind of code (`handbook/guidelines/idiomatic-code-checklist.md`),
and each lint's `lint.toml` names one in its `rule` field.

## Precision

### R1: Idiomatic code gets no warnings

A lint is silent on the corpus `spec.cp` files, `std/` and the practice
solutions; a finding there is a bug in the lint.
Reason: a lint that cries wolf teaches users to ignore every lint.
Example: L0203 firing on a port's `@check` fn local is fixed in the port
only after its owner agrees the shadow is a real defect.
Enforced by: `crates/cpl-lint/src/corpus_tests.rs` (`corpus_warnings_only_go_down`)
and the std warning baseline.

### R2: Unsure means silent

When a rule can't be sure the shape is a defect, it stays silent or warns
without a fix.
Reason: a wrong warning costs more than a missed one.
Example: `sort_for_extreme` stays quiet when the sorted copy is read twice.
Enforced by: each record's `near_miss.cp` (`crates/cpl-lint/src/record_tests.rs`),
and `newcomer_findings_are_the_ones_their_comments_expect` in
`crates/cpl-lint/src/corpus_tests.rs`, which grades every finding on the
newcomer programs against their `// expect:` comments.

### R3: Every look-alike is a near miss

Every shape that looks like the trigger but must stay quiet is in the
record's `near_miss.cp`, including each false positive ever reported.
Reason: a fixed false positive must not come back.
Example: `push_loop/near_miss.cp` has a loop that pushes only some items.
Enforced by: `crates/cpl-lint/src/record_tests.rs`; review.

## Facts

### R4: Facts come from the checker

Types, resolved names, ranges and effects come from the checker's analysis
(`cpl_lint::Facts`); a lint never infers them from a declaration, a name or
a literal. A missing fact is asked of the compiler domain.
Reason: a second type guesser drifts from the checker and the editor.
Example: `float_equality` reads the checker's type of `d`, not the word
`sqrt` in its initializer.
Enforced by: the `lint_type_syntax` and `consumer_rederived` ratchets in
`scripts/debt-ratchet.py`; the first names each rule line that reads a
written type.

### R5: Match what a name resolves to

A rule about a std item checks the module the name resolves to, never its
spelling, so a file's own `fn sum` is never flagged.
Reason: users may name their functions anything.
Example: `free_fold` skips `sum(xs)` when the file declares `fn sum`.
Enforced by: the `lint_std_names` ratchet in `scripts/debt-ratchet.py`.

### R6: Don't restate the checker

If a checker error covers the shape, the fix goes on that error and the
lint is retired; a lint never fires at the span of a checker diagnostic.
Reason: one cause, one message.
Example: `by_twin` was retired once the checker's E0201 offered `xs.min(by: f)`.
Enforced by: the `lint_checker_overlap` ratchet (`missing` until it lands).

### R7: A file that doesn't check is never lint-clean

`cpl lint` on a file with checker errors reports those errors and never
says the file is clean.
Reason: "0 warnings" on a broken file is a lie.
Example: `cpl lint` on a file with an E0302 exits 1 and prints the error.
Enforced by: `crates/cpl-cli/tests/` lint snapshots.

## Fixes

### R8: A fix clears its lint and still parses

Applying a lint's fix to `before.cp` removes that lint and gives a file
that parses.
Reason: a fix that leaves the warning is noise.
Example: `push_loop`'s fix gives a comprehension, and the warning is gone.
Enforced by: `every_record_fix_clears_its_lint_and_still_parses` in
`crates/cpl-lint/src/record_tests.rs`.

### R9: A fix adds no checker error

What a fix writes has no checker error the original lacked, and `cpl fix`
applies a fix only when the result re-checks clean.
Reason: fixes are applied without reading them.
Example: `sort_for_extreme` writes `xs.min()!`, since `.min()` is optional.
Enforced by: `every_record_fix_adds_no_checker_error` in
`crates/cpl-lint/src/record_tests.rs`.

### R10: A fix converges and is formatted

Fixing the fixed file again changes nothing, and what a fix writes is
already in `cpl fmt` form.
Reason: one `cpl fix` run is the whole job; no follow-up edit.
Example: `float_sqrt_for_int` rewrites `floor(sqrt(n))` to `n.isqrt`, and a
second run finds nothing.
Enforced by: `every_record_fix_converges_and_is_formatted` in
`crates/cpl-lint/src/record_tests.rs`.

### R11: A fix keeps what the program prints

A fixed program prints the same output on the same input, and keeps CRLF
line ends.
Reason: a style fix must not change the answer.
Example: `push_loop`'s `before.cp` and `after.cp` print the same list.
Enforced by: `crates/cpl-lint/src/fix_tests.rs`; on every corpus file: `missing`.

### R12: Intent-dependent edits get help, not a fix

When the right edit depends on what the user meant, the lint gives help
text and no fix, and `lint.toml`'s `no_fix` says why.
Reason: guessing intent breaks working code.
Example: `state_above_multitest` can't know which state is per test.
Enforced by: `crates/cpl-lint/build.rs` reads `no_fix`; review.

## Messages

### R13: The message names the fact

The message says what the code does, in the user's words.
Reason: the user must recognise their own code in it.
Example: "a loop that only pushes onto `doubled` is a comprehension".
Enforced by: the record tests' message snapshots; review.

### R14: The help gives the exact replacement

The help gives the exact CP Lang spelling and the reason in one line, using
the spec's vocabulary (`spec/GLOSSARY.md`) and only names that exist.
Reason: a fix the user can type without looking anything up.
Example: "write `[2 * x for x in raw]`; the comprehension says what each item is".
Enforced by: R9's test checks the spelling exists; review for vocabulary.

### R15: Every lint has an explain page

Every lint has an `explain.md` that `cpl explain <code>` prints, with its
trigger, its fix and the handbook rule it enforces.
Reason: a warning is a lesson only if it can be looked up.
Example: `cpl explain L0507`.
Enforced by: `crates/cpl-lint/build.rs` requires the record files.

### R16: One finding per cause

Two lints never teach one shape, and nested findings are one cause
reported once or distinct causes listed separately.
Reason: two warnings for one mistake look like two mistakes.
Example: `sort_for_extreme` and `copy_sort_writeback` never fire on the same span.
Enforced by: `missing`.

## Records and configuration

### R17: Every lint enforces a written rule

A lint's `lint.toml` names the checklist rule it enforces in `rule`, as
`<topic>/R<n>`, and `handbook/guidelines/<topic>-checklist.md` defines it.
Reason: a lint with no rule behind it is a one-off; either write the rule or
drop the lint.
Example: `sort_for_extreme` cites `idiomatic-code/R3`: use std for a
standard algorithm.
Enforced by: `crates/cpl-lint/build.rs` (`check_rule`) fails the build.

### R18: The record is complete

Each lint has `lint.toml`, `before.cp` (fires once), `after.cp` (passes
`cpl check`), `near_miss.cp` and `explain.md`, and a row in the lint table
of `spec/diagnostics.md`.
Reason: the record is the lint's spec and its test.
Example: `spec/lints/push_loop/`.
Enforced by: `crates/cpl-lint/build.rs` and `crates/cpl-lint/src/record_tests.rs`.

### R19: Every lint is easy to turn off

Every lint has a name and a group (`correctness`, `perf`, `idiom`, `style`,
`pedantic`); one line turns either off, in `cp.toml [lints]` or with
`with lint(name: allow)`.
Reason: users decide which advice they want.
Example: `push_loop = "allow"` or `style = "allow"` in `cp.toml`.
Enforced by: `crates/cpl-lint/src/tests.rs` config tests.

### R20: Codes are stable

A lint code is never reused; a retired lint keeps its code reserved.
Reason: old suppressions and links must not change meaning.
Example: L0510 stays reserved after `by_twin` was retired.
Enforced by: `crates/cpl-lint/build.rs` (duplicate codes); reuse: review.

## Speed

### R21: One pass, shared analysis

A rule is one pass over the tree and the facts, and every tool lints the
analysis its warm checker database already holds.
Reason: lint runs on every keystroke in the editor.
Example: the language server lints the database it already has for hover.
Enforced by: the `own_databases` ratchet in `scripts/debt-ratchet.py`.

### R22: Formatting is idempotent and keeps meaning

`cpl fmt` run twice changes nothing the second time, and a formatted file
checks and runs exactly as the original did.
Reason: a formatter that changes behaviour, or never settles, can't run on save.
Example: formatting a corpus port leaves its `cpl check` codes and its output on
the sample input unchanged.
Enforced by: `scripts/fmt_check.py` over every tracked `.cp` file;
idempotence on broken files by `mutated_sources_never_crash_lint_or_fmt` in
`crates/cpl-lint/src/fuzz_tests.rs`.

## Robustness

### R23: No input crashes lint or fmt

Lint and fmt answer every file, however broken, without panicking.
Reason: the editor runs both on every keystroke over half-typed code; a
crash there loses the user's diagnostics and formatting.
Example: `cnt[]x = 1`, a key deleted mid-edit, gets a checker error, not a
crash.
Enforced by: `mutated_sources_never_crash_lint_or_fmt` in
`crates/cpl-lint/src/fuzz_tests.rs` (deeper runs with `CPL_FUZZ_MUTANTS`).

### R24: Lint and fmt stay fast in the editor

Linting an analysed file, and formatting it, each take at most a tenth of the
500 ms front-end budget on the largest contest programs we have.
Reason: the editor runs both on every keystroke, after the checker; their
time comes straight out of the wait before diagnostics appear.
Example: the largest corpus port lints and formats in a few milliseconds each
in an optimized build.
Enforced by: `lint_and_fmt_of_the_largest_files_fit_a_tenth_of_the_front_end_budget`
in `crates/cpl-lint/src/speed_tests.rs` (asserted under `cargo test --release`).

### R25: A lint fires wherever its pattern is written

A rule finds its pattern with any names and inside any program: renaming
what the example binds, or planting it after other code, changes nothing.
Reason: contestants pick their own names and write the pattern deep in a
long file, never as the record's few lines alone.
Example: `loop_invariant_mod_div` fires on `xs2[i] /= d2` after a whole
corpus solution just as on `xs[i] /= d` alone.
Enforced by: `a_record_fires_the_same_renamed_and_planted_in_a_clean_program`
in `crates/cpl-lint/src/plant_tests.rs`.

### R26: An idiom with evidence gets a lint

When a real program (a newcomer draft, an acceptance run) writes a pattern
CP Lang has a better spelling for, a lint teaches it, or the spot is
recorded as owed, or as something no lint can decide, with its owner.
Reason: users repeat the hand-written version until something points them
to the better one; a pattern seen and not recorded is seen again.
Example: acceptance runs wrote prefix sums with a push loop four times;
`corpus/newcomer/acceptance.txt` records them as `missing:prefix-totals-list`
until `Prefix` can be walked whole.
Enforced by: the `idioms_evidenced_unlinted` ratchet in
`scripts/debt-ratchet.py`, over `// expect: missing` marks in
`corpus/newcomer` and `corpus/newcomer/acceptance.txt`.
