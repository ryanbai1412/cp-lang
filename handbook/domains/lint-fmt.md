# Lint and fmt

| | |
|---|---|
| Kind | Domain charter |
| Owner | lint-fmt |
| Status | Active |
| Describes | `crates/cpl-lint/`, `crates/cpl-fmt/`, `spec/lints/`, `spec/tools/fmt-lint.md` (with spec), `cpl lint`, `cpl fmt`, the removed-syntax codemods behind `cpl fix`, `scripts/fmt_check.py`, `scripts/new-lint.py` |

`cpl fmt` gives every CP Lang file one layout and never changes what the
program means. `cpl lint` points at legal code that is probably wrong, slow
or not the CP Lang way, explains why, and offers fixes that are safe to apply
without reading them. If you have saved a file in an editor or run
`cpl fix`, you have used this domain's work.

## What good looks like

A contestant fresh from C++ writes

```
var doubled: List<Int> = []
for x in raw: doubled.push(2 * x)
```

and sees, in the editor and in `cpl lint` alike,

```
warning[L0507]: a loop that only pushes onto `doubled` is a comprehension
  --> main.cp:2:1
  = help: write `[2 * x for x in raw]`; the comprehension says what each item is, so no push is missed or doubled
```

One `cpl fix` rewrites it to `let doubled = [2 * x for x in raw]`, already
formatted, printing the same output. On idiomatic code (the corpus
solutions, `std/`) neither tool says or changes anything, and saving a file
moves only the lines that were edited.

## How it fits together

- **One analysis.** The checker (`cpl-check`) owns what a file means. Every
  tool (`cpl lint`, `cpl fix`, the language server, the browser playground,
  `cpl contest`'s watch loop) lints the analysis its checker database
  already holds, through `cpl_lint::lint_analysis`. A lint reads the
  checker's facts (types, resolved names, ranges) and never guesses them
  from syntax.
- **Records are the spec.** Each lint is a folder in `spec/lints/<name>/`:
  `lint.toml` (code, level, group, the handbook rule it enforces),
  `before.cp`, `after.cp`, `near_miss.cp` and `explain.md`. The lint
  registry is generated from them, and the same files are the lint's tests.
- **Fixes are verified.** `cpl fix` applies a fix only when the result checks
  clean, and record tests check each fix clears its lint, converges and is
  formatted.
- **The formatter is token-preserving.** `cpl-fmt` lays out the lossless
  syntax tree and may only add or drop trailing commas; `cp.toml [fmt]`
  sets line and indent width.

## How we know it is good

Every record runs as a test: `before.cp` fires once, `near_miss.cp` stays
quiet, and the fix clears the lint, adds no error and converges. The corpus
and `std` warning counts only go down. `scripts/fmt_check.py` formats every
tracked file and checks the result is stable and checks and runs the same.
Parity tests check that the editor, the playground and `cpl lint` report the
same findings.

The weakest areas today: nothing measures how much a small edit
reformats, the suites have no mutation score, and only a few dozen corpus
programs have a fix that applies, so the corpus run of fixes is thin. Those rows are marked `missing` in the quality
register below and are built first.

## Where to start

- **Add or change a lint:** `handbook/guidelines/lints.md`, then
  `python3 scripts/new-lint.py`.
- **A lint fired on correct code:** add it to that record's `near_miss.cp`
  and fix the rule in `crates/cpl-lint/src/rules/`.
- **A formatting bug:** add the input to `crates/cpl-fmt/src/tests.rs` and
  run `python3 scripts/fmt_check.py`.

# Reference

## Scope

| Owns | Does not own (owner) |
|---|---|
| The lint engine: levels, groups, `with lint(name: allow)`, `cp.toml [lints]`, fix application (`apply_fixes`, `fix_in`) | Which idioms exist and their prose (`spec/idioms/`, idioms) |
| Every `L02xx`-`L05xx` rule and its record under `spec/lints/<name>/` (`lint.toml`, `explain.md`, `before.cp`, `after.cp`, `near_miss.cp`) | Checker lints `L06xx` and bound lints `L03xx` over declared ranges (diagnostics) |
| `cpl lint`, `cpl lint --fix`, `--list`; the removed-syntax codemods in `cpl fix` | The fix applier for checker errors, `cpl_ir::fix` (diagnostics) |
| The formatter (`crates/cpl-fmt`), `cpl fmt`, `--check`, range formatting, `cp.toml [fmt]` | The parser, lossless tree and checker facts that lints read (compiler) |
| The corpus-wide fmt gate (`scripts/fmt_check.py`) and the lint precision ratchets | How the editor shows a finding or formats on save (tools); the playground UI (site) |
| `handbook/guidelines/lints.md` | Lint codes' normative table in `spec/diagnostics.md` (diagnostics, with spec) |

## Interfaces

**Provides**

| Interface | Used by | Held by |
|---|---|---|
| `cpl_lint::lint_analysis(analysis, config)`: the one lint entry, on the analysis a tool's warm checker database already holds | `cpl lint`, `cpl fix`, `cpl-lsp`, `cpl-wasm`, the conformance runner, `cpl contest` watch | `crates/cpl-lint/src/corpus_tests.rs` `the_cli_and_the_editor_read_the_same_facts`, `crates/cpl-wasm/tests/parity.rs` `the_playground_lints_like_cpl_lint` |
| `cpl_lint::fix_in`, `apply_lint_fixes`, `apply_fixes` | `cpl fix`, the editor's fix-all, the playground | `crates/cpl-cli/tests/fix_behavior.rs`, `crates/cpl-lint/src/record_tests.rs` |
| `cpl_lint::LINTS`, generated from `spec/lints/*/lint.toml` | `cpl explain L0xxx`, `cpl lint --list`, the site's lint pages | `crates/cpl-lint/src/record_tests.rs` `every_lint_matches_the_spec_lint_list` |
| `cpl_fmt::format`, `format_with`, `format_range`, `format_on_type`, `Style` from `cp.toml [fmt]` | `cpl fmt`, `cpl-lsp` formatting, on-type and range formatting | `crates/cpl-fmt/src/tests.rs`, `crates/cpl-fmt/src/range_tests.rs`, `crates/cpl-lsp/src/tests/format.rs` |

Invariants every consumer may rely on: fmt never changes the token stream
except trailing commas and is idempotent; a lint code never changes meaning
and is never reused; lint never blocks `cpl build`; a file that does not
check is never reported lint-clean; no tool builds a checker database just
to lint.

**Needs**

| From | What |
|---|---|
| compiler | Checker facts (types, resolved names, ranges, effects) and diagnostics with fixes, so a lint never re-derives them from syntax; one session API (`cpl_check::Session`) |
| diagnostics | Checker codes that carry the CP Lang fix, so a lint that repeated them can be retired |
| stdlib, idioms | `std` and corpus solutions that are idiomatic, so a finding there is a lint bug |
| tools | Editors that show lint findings and format on save exactly as the CLI does |
| spec | `spec/tools/fmt-lint.md` and one vocabulary for messages |

## Quality register

Each row is a quality a contestant, learner, editor user, library author or
contributor feels, the number that measures it, and the strongest mechanism
that enforces it. Mechanisms rank: a design that makes the failure
impossible, then a generated, differential, property or mutation suite, then
a hand-written suite, then a lint, ratchet or gate, then a process.
`missing` marks a row with no real check yet. A row about how lints are
written names its rule in `handbook/guidelines/lints-checklist.md`
(`lints/R1` and on), which states the rule; the row only measures it. Targets and current values are
in `plan/domains/lint-fmt/metrics.toml`.

### Lint precision: it never cries wolf

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| No warnings on idiomatic code (`lints/R1`) | warnings on corpus `spec.cp`, `std`, the practice solutions | ratchet, only goes down | `crates/cpl-lint/src/corpus_tests.rs` `corpus_warnings_only_go_down`, `crates/cpl-lint/src/std_tests.rs` |
| A rule's near misses stay quiet (`lints/R3`) | records whose `near_miss.cp` or `after.cp` fires | generated suite over every record | `crates/cpl-lint/src/record_tests.rs` `every_lint_record_fires_on_before_only` |
| A lint never repeats a checker error at the same place (`lints/R6`) | lint findings at the span of a checker diagnostic | design (rules read checker facts) plus suite | `crates/cpl-lint/src/checker_tests.rs` `a_lint_does_not_repeat_the_error_at_its_place` |
| A user's own name is never mistaken for a std item (`lints/R5`) | rules matching std items by spelling | ratchet `lint_std_names`, to 0 by matching resolved paths | `scripts/debt-ratchet.py` |
| Lints read the checker's types, never guess them from syntax (`lints/R4`) | rule lines that read a written type or compare type names | ratchets `lint_type_syntax` (each line named, per rule) and `consumer_rederived` | `scripts/debt-ratchet.py` |
| One cause, one finding (`lints/R16`) | nested or duplicated findings for one root cause | hand-written suite | `crates/cpl-lint/src/corpus_tests.rs` `nested_findings_are_one_cause_or_listed_distinct` |
| Precision on code we did not write (`lints/R2`) | findings on `corpus/newcomer` that no `// expect:` comment in the file names, and named lints that stay silent | hand-graded suite | `crates/cpl-lint/src/corpus_tests.rs` `newcomer_findings_are_the_ones_their_comments_expect` |

### Lint recall and teaching value

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| C++, Python and Rust habits get the CP Lang spelling | marked spots flagged on the newcomer set and skill-eval first tries | generated suite | `crates/cpl-cli/tests/newcomer.rs`, `crates/cpl-lint/src/idiom_tests.rs` |
| Every idiom with evidence has a rule (`lints/R26`) | newcomer `missing` marks, and acceptance findings with no lint, no owed mark or no owner | ratchet `idioms_evidenced_unlinted` | `scripts/debt-ratchet.py`, `corpus/newcomer/acceptance.txt` |
| A lint catches the bug it names (`lints/R25`) | records whose `before.cp`, renamed and planted after a clean corpus program, fires a different count, or whose planted `near_miss.cp` fires | metamorphic suite over every record | `crates/cpl-lint/src/plant_tests.rs` |
| The most frequent lints are worth seeing | hits per lint on corpus `port.cp`; top lints sampled for true positives | process, ranked in metrics | `plan/domains/lint-fmt/metrics.toml` |
| A finding teaches in one read: the fact, the CP Lang way, a link | findings without a help or an explain page | gate | `crates/cpl-lint/src/record_tests.rs`, `crates/cpl-lint/src/checker_tests.rs` `every_checker_lint_has_a_registry_entry_for_cpl_explain` |
| Help text names things that exist (`lints/R14`) | helps naming a missing function, method or option | generated suite | `crates/cpl-lint/src/record_tests.rs` `every_help_names_real_vocabulary` |
| One spelling per idea across lint, spec and errors | banned spellings in records and helps | gate: `spec/GLOSSARY.md` bans over every record; option spellings over every help | `scripts/spec-vocab.py`, `crates/cpl-lint/src/record_tests.rs` `records_and_helps_spell_options_some_none` |

### Fixes you can apply blind

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| A fix clears its lint and still parses (`lints/R8`) | record fixes that leave the lint or a parse error | generated suite | `crates/cpl-lint/src/record_tests.rs` `every_record_fix_clears_its_lint_and_still_parses` |
| A fix never adds a checker error (`lints/R9`) | fixed files with more errors, over records, `std` and corpus | generated suite | `crates/cpl-lint/src/record_tests.rs` `every_record_fix_adds_no_checker_error`, `crates/cpl-lint/src/corpus_tests.rs` `fixes_never_add_checker_errors` |
| A fix keeps what the program prints (`lints/R11`) | runnable programs whose output changes after `cpl fix` | differential suite, plus a corpus run | `crates/cpl-cli/tests/fix_behavior.rs`, `crates/cpl-lint/src/behaviour_tests.rs`; `cpl-conform rewrites` (`#fix` cases) |
| The fixed file is what a person would write: formatted, no churn outside the edit (`lints/R10`) | fixed files `cpl fmt --check` rejects; lines changed outside the fix | generated suite over every record, plus a corpus run: a fix keeps a formatted file formatted | `crates/cpl-lint/src/record_tests.rs` `every_record_fix_converges_and_is_formatted`; `cpl-conform rewrites` (`#fix` cases) |
| Fixes converge: a second `cpl fix` changes nothing (`lints/R10`) | non-idempotent fix runs | generated suite over every record | `crates/cpl-lint/src/record_tests.rs` `every_record_fix_converges_and_is_formatted` |
| Every lint has a fix or says why it has none (`lints/R12`) | lints with neither | gate | `crates/cpl-lint/src/record_tests.rs` `every_lint_has_a_fix_or_says_why_not` |
| `cpl fix` migrates removed syntax so old solutions keep building | codemods whose output fails `cpl check` | hand-written suite | `crates/cpl-lint/src/tests.rs`, `crates/cpl-lint/src/fix_tests.rs` |
| Fixes keep CRLF and non-ASCII sources intact | fixes that change line ends or split a character | hand-written suite | `crates/cpl-lint/src/record_tests.rs` `every_record_fix_keeps_crlf` |

### Configuration and suppression

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| One line turns a lint or a group off: `cp.toml [lints]` or `with lint(name: allow)` | lints and groups that ignore either | hand-written suite | `crates/cpl-lint/src/checker_tests.rs` `with_lint_allows_a_checker_lint_by_name`, `cp_toml_sets_a_checker_lint_level`, `crates/cpl-lint/src/tests.rs` |
| A misspelt lint name is told, with the nearest name | silent unknown names | hand-written suite | `crates/cpl-lint/src/checker_tests.rs` `a_misspelt_checker_lint_suggests_its_name` |
| Every tool honours the same levels and `with lint` regions | CLI and editor findings differing under a config | differential suite | `crates/cpl-cli/tests/lsp_parity.rs` |
| Library authors declare their own lints with `@lint`, without Rust | a user-library `@lint` that fires and fixes | `missing` | — |

### Fmt safety

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| Formatting never changes meaning | files whose tokens, parse or checker result change | design (`verify` re-lexes the output and rejects any token change) plus corpus suite | `crates/cpl-fmt/src/lib.rs` `verify`, `crates/cpl-fmt/src/tests.rs` `formats_the_whole_corpus`, `scripts/fmt_check.py` |
| Formatting is idempotent | files that change on a second run | corpus gate | `scripts/fmt_check.py`, `scripts/test_fmt_check.py` |
| Formatted programs still print the same | programs whose output differs after `cpl fmt` | corpus run on every test input (token equality implies it too) | `cpl-conform rewrites` (`#fmt` cases) |
| Every tracked `.cp` file is formatted | files `cpl fmt --check` would change | gate | `scripts/check.sh` fmt step, `scripts/fmt_check.py` |
| Files that don't parse still format, and unfinished statements are left alone | panics or edits inside a broken statement | hand-written suite | `crates/cpl-fmt/src/range_tests.rs` `an_unfinished_statement_is_left_alone` |
| No input crashes fmt or lint (`lints/R23`) | panics and non-idempotent fmt on mutated sources | generated suite: token mutations of `std`, corpus `spec.cp` and every record, fixed seed | `crates/cpl-lint/src/fuzz_tests.rs` `mutated_sources_never_crash_lint_or_fmt` |

### Fmt output quality

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| Output is what a careful person writes | style cases from the spec | hand-written suite | `crates/cpl-fmt/src/tests.rs` |
| A revision reformats only what it touched | files where `cpl fmt` changes more than a one-line spacing edit, over ports, specs and `std` | generated suite (spacing edits; token edits not yet) | `crates/cpl-fmt/src/churn_tests.rs` `a_one_line_spacing_edit_reformats_only_that_line` |
| Formatting a range matches formatting the file there | statements whose range format differs from the whole-file format | generated suite over newcomer files | `crates/cpl-fmt/src/range_tests.rs` `every_line_of_the_newcomer_files_formats_alone` |
| Opinionated defaults, few knobs: line width and indent in `cp.toml [fmt]` | style options beyond the spec's list | hand-written suite | `crates/cpl-fmt/src/tests.rs` `cp_toml_sets_the_indent_and_line_widths`, `a_style_changes_only_the_widths` |
| Comments and blank lines stay where they were | comments moved or dropped | hand-written suite | `crates/cpl-fmt/src/tests.rs` `keeps_comments_where_they_are` |
| Docs and site snippets are formatted the same way | snippets `cpl fmt --check` would change, in the handbook, spec, docs, std notes and site | gate | `crates/cpl-fmt/src/docs_tests.rs` `every_doc_snippet_is_formatted` |

### Every tool, every platform

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| The editor shows exactly the CLI's lint findings and fixes | files whose findings differ | differential suite | `crates/cpl-cli/tests/lsp_parity.rs`, `crates/cpl-lsp/src/tests/parity.rs` |
| The playground lints like `cpl lint` | records whose findings differ | differential suite | `crates/cpl-wasm/tests/parity.rs` `the_playground_lints_like_cpl_lint` |
| Format on save does what `cpl fmt` does, following `cp.toml [fmt]` | editor results differing from `cpl fmt` | hand-written suite plus editor CI jobs | `crates/cpl-lsp/src/tests/format.rs`, `.github/workflows/ci.yml` jobs `neovim`, `vscode`, `jetbrains` |
| Windows checkouts stay CRLF | files whose line ends change | hand-written suite | `crates/cpl-fmt/src/tests.rs` `keeps_crlf_line_endings_of_a_windows_checkout`, `crates/cpl-fmt/src/range_tests.rs` `keeps_crlf_line_ends` |
| A file that fails `cpl check` is never reported lint-clean | files printed clean with errors | hand-written suite | `crates/cpl-lint/src/checker_tests.rs` `a_file_that_does_not_check_is_not_reported_clean` |

### Catalogue and metadata

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| Every lint is a record with before, after, near miss and explanation | lints without a complete record | design (registry generated from records) plus gate | `crates/cpl-lint/build.rs`, `crates/cpl-lint/src/record_tests.rs` `every_lint_record_directory_belongs_to_a_lint` |
| Records and `spec/diagnostics.md` agree | codes in one and not the other | gate | `crates/cpl-lint/src/record_tests.rs` `every_lint_matches_the_spec_lint_list`, `crates/cpl-lint/src/checker_tests.rs` `checker_lints_match_the_spec_table` |
| A record's `after.cp` is real CP Lang | records whose `after.cp` fails `cpl check` | gate | `crates/cpl-lint/src/record_tests.rs` `every_record_after_passes_cpl_check` |
| No two lints teach one shape | duplicate records | gate | `crates/cpl-lint/src/record_tests.rs` `no_two_records_teach_one_shape_undecided`, `record_codes_are_unique` |
| A retired code is never reused | reused codes | process: append-only table; `missing` as a check | — |
| Adding a lint is cheap | files touched and minutes to a green new lint | scaffold | `scripts/new-lint.py`, `scripts/test_new_lint.py` |

### Speed and architecture

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| Lint and fmt add little to the editor's response (`lints/R24`) | lint and fmt time on the largest ports, against a tenth of the 500 ms front-end budget; the largest `std` modules printed | gate in optimized builds | `crates/cpl-lint/src/speed_tests.rs` `lint_and_fmt_of_the_largest_files_fit_a_tenth_of_the_front_end_budget` |
| One analysis: no tool re-checks a file to lint it | checker databases built outside the session | ratchet `own_databases` | `scripts/debt-ratchet.py` |
| Lint knows no std API by name | std names in rules | ratchet `lint_std_names` | `scripts/debt-ratchet.py` |
| Suites catch regressions in the rules | mutation score of `cpl-lint` and `cpl-fmt` suites | `missing` | — |
| Every escaped lint or fmt bug becomes a case | escaped bugs without a record near miss or test | process: a false positive lands as a `near_miss.cp` case | `spec/lints/*/near_miss.cp` |

## Suites and tools

| Tool | What it catches | Run it |
|---|---|---|
| Lint records | Lints firing on `after.cp` or near misses; fixes that don't clear, don't check or lose CRLF; spec table drift | `cargo test -p cpl-lint --lib record_tests` |
| Corpus and std ratchets | New warnings on idiomatic code; fixes that add errors | `cargo test -p cpl-lint --lib corpus_tests std_tests` |
| Checker-lint tests | Levels, `with lint`, misspelt names, repeated checker errors | `cargo test -p cpl-lint --lib checker_tests` |
| Fix behaviour | Fixes that change program output | `cargo test -p cpl-cli --test fix_behavior` |
| Editor and playground parity | Findings differing between CLI, LSP and wasm | `cargo test -p cpl-cli --test lsp_parity`, `cargo test -p cpl-wasm` |
| Fmt unit and range suites | Style regressions, range format drift, CRLF | `cargo test -p cpl-fmt`, `cargo test -p cpl-lsp format` |
| Corpus fmt gate | Meaning changes, non-idempotence, unformatted files | `python3 scripts/fmt_check.py` |
| New lint scaffold | A record skeleton with all five files | `scripts/new-lint.py <name>` |
| `cpl lint --list`, `cpl explain L0507` | Every lint and its page | `cpl lint --list` |

## Changes

- 2026-11-23: first version, folded from `plan/domains/lint-fmt/README.md` and `ARCHITECTURE.md`.
