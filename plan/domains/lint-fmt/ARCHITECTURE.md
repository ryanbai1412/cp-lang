# lint-fmt architecture

The code as it stands on main (read 2026-11-22). The owning lane keeps this
current in the same commit as any structural change. Where v0.3 goes:
[design/overview.md](design/overview.md).

## Modules

### `crates/cpl-fmt` (1,804 lines with tests)
Depends on `cpl-syntax` (lexer, parser) and `cpl-diag` only.

| File | Lines | What it is for |
|---|---|---|
| `src/lib.rs` | 142 | `format`, `is_formatted`, `FormatError`; `INDENT = 4`, `MAX_WIDTH = 100`; the final `verify` step |
| `src/spacing.rs` | 345 | Space or no space between each pair of adjacent tokens |
| `src/layout.rs` | 622 | Indentation, blank lines, comments, breaking an over-wide bracket one element per line with a trailing comma (`MIN_BREAK_WIDTH`) |
| `src/imports.rs` | 85 | Sorts each run of one-line `use` lines at one indentation, `std` paths first, with its own verify (only whole lines move) |
| `src/cli.rs` | 154 | `cpl fmt [--check] <files>` |
| `src/tests.rs` | 541 | Unit cases and `formats_the_whole_corpus` (AST equal after formatting) |

The formatter works on the **token stream**, not on a parsed tree: it lexes,
re-emits tokens with canonical whitespace, and `verify` re-lexes the output and
requires the same tokens (trailing commas excepted). So it formats files that
do not parse, and cannot reorder anything (no `use` sorting, no moving `with`
lines) without a new verification rule.

### `crates/cpl-lint` (14,702 lines with tests)
Depends on `cpl-syntax`, `cpl-diag`, and, outside wasm only, `cpl-check` and
`cpl-prover` (`Cargo.toml`, `cfg(not(target_arch = "wasm32"))`).

| File | Lines | What it is for |
|---|---|---|
| `build.rs` | | Generates the `LINTS` registry (one const per `spec/lints/<name>/`, code order) from each record's `lint.toml` (code, level, title, summary) and its `explain.md`, `before.cp`, `after.cp`, `near_miss.cp` |
| `src/lib.rs` | 704 | `Level`, `Lint`, `include!` of the generated registry; `lint` (source only, for tests), `fix_in`, `apply_fixes`; `with lint(name: allow)` regions; removed-syntax codemods behind `cpl fix`; the `Proven*` analysis API |
| `src/checker.rs` | 74 | `CHECKER_LINTS`: the names of L0601-L0609, which `cpl-check` emits; `checked_analysis` sorts what `cpl check` reported into checker lints at their levels and `with lint` regions, and the errors lints can't see past |
| `src/config.rs` | 153 | `Config` from `cp.toml [lints]` (levels by name) plus `memory_mb` |
| `src/walk.rs` | 513 | Tree walkers over `cpl_syntax` items shared by the rules |
| `src/rules/mod.rs` | 34 | `run`: calls each rule module in turn |
| `src/rules/proven/mod.rs`, `cost.rs` | 1,889, 238 | the `Proven*` analyses (value ranges, indexes, loops, sizes, per-fn iteration counts) the CLI and LSP read; checker facts, to move to `cpl-check` |
| `src/rules/bounds.rs` | 1,089 | Value ranges from declared input bounds; a second copy of what the checker proves |
| `src/rules/taste/mod.rs` | 1,890 | L05xx idiom lints over statement sequences (search loop, folds, push loops, ...) each as its own pattern match |
| `src/rules/taste/{search,range_tree,string_algo,stdlib_algo}.rs` | 237, 501, 619, 1,199 | the `hand_rolled_*` and `sorted_range_method` matchers, one family per file |
| `src/rules/literals.rs` | 110 | L0506 repeated big literals, skipping input bounds |
| `src/rules/mod_div.rs` | 991 | L0415 loop-invariant `Mod` division |
| `src/rules/types.rs` | 490 | L0204-L0207 spelling lints |
| `src/rules/scopes.rs` | 291 | L0201, L0203 |
| `src/rules/lists.rs` | 244 | L0414 unproven list index |
| `src/rules/floats.rs` | 177 | L0301 float equality, by reading the source (no types) |
| `src/rules/fields.rs` | 170 | L0202 unread field, by name |
| `src/rules/imports.rs` | 139 | L0208 redundant prelude import |
| `src/rules/memo.rs` | 128 | L0413 memo key space vs `memory_mb` |
| `src/cli.rs` | 250 | `cpl lint [--fix] [--list]`, summary line |
| `src/tests.rs` | 2,384 | Per-lint fire / no-fire / fix cases |

**Pipeline.** `lint(src, config)` parses; a file with syntax errors is not
linted. Rules run over the parsed items and return `Finding { lint, diag }`;
`lint_file` drops allowed ones, sets severity from the level, sorts by span
and dedups. `fix_at` runs removed-syntax rewrites, then checker-backed ones
(non-wasm), then applies lint fixes to a fixed point; `apply_fixes` takes
the first fix of each diagnostic and skips overlapping edits.

## Interfaces other domains use
Found with `grep -rhoE 'cpl_(lint|fmt)::[A-Za-z_]+' crates --include=*.rs`.

| Interface | Consumers |
|---|---|
| `cpl_fmt::format`, `cpl_fmt::cli` | `cpl-cli` (`cpl fmt`), `cpl-lsp` (format document) |
| `cpl_lint::lint_analysis`, `checked_analysis`, `Config` | the one lint entry: `cpl lint`, `cpl fix`, `cpl-lsp`, `cpl-wasm`, `cpl-conform` runner, `cpl-contest` watch, each on the analysis its warm `Database` holds |
| `cpl_lint::fix_in`, `apply_lint_fixes`, `apply_fixes` | `cpl fix`, `cpl lint --fix`, the editor's fix-all, the playground's fix |
| `cpl_lint::LINTS` | `cpl-cli` (`cpl explain L0xxx`, `--list`) |
| `Proven{Expr,Loop,Fn,Index,Size}`, `Run`, `proven_*_parsed`, `domain_sizes_parsed`, `list_sizes` | `cpl-lsp` inlay hints and hovers |

One entry (ARCHITECTURE.md B6 #2): a tool lints through `lint_analysis` on
the analysis it already has and never builds a `Database` for lint. When
`cpl_check::Session` lands it becomes `cpl_lint::lint(&session, file)`;
`lint(src)` stays a test helper.

The `Proven*` API is the checker's job living in the wrong crate; it moves to
`cpl-check` in M3 and `cpl-lsp` follows (request in the compiler worklog).

## Invariants
- Fmt never changes tokens except trailing commas (`verify` in `src/lib.rs`);
  it is idempotent. Corpus-wide this is held by `formats_the_whole_corpus` and,
  once `devin/wip-fmt` lands, `scripts/fmt_check.py` in the fast gate.
- Fmt has no options: one style.
- Lint never blocks `cpl build`; it only reports. Syntax errors mean no lints.
- Every lint has a stable code and name in `LINTS`; codes never get reused.
- A fix is offered only where the rule is sure; `apply_fixes` never applies
  two overlapping edits.

## Known debt
- **One meaning change.** Formatting `crates/cpl-cli/tests/fixes/c_operators.cp`
  turns its E0107 into E0102 (`python3 scripts/fmt_check.py`, from
  `devin/wip-fmt`). Token-equal output that the parser reads differently.
- **Duplicated analysis.** `rules/bounds.rs` and the `Proven*` passes in
  `src/lib.rs` re-derive ranges the checker proves; `cpl-prover` is called
  from the linter.
- **Hard-coded stdlib names.** 16 distinct type names as string literals in
  `src/rules/*.rs` (`"Prefix"`, `"SortedSet"`, `"Mod"`, ...), against the spec's
  "no stdlib privilege".
- **Browser lints differ.** The checker-backed lints and fixes are compiled
  out on wasm (8 `target_arch = "wasm32"` gates: `Cargo.toml` 1, `src/lib.rs` 2, `src/rules/bounds.rs` 5).
- **Metadata drifts.** `LINTS` and `spec/diagnostics.md` disagree: L0601-L0609
  are specified and not implemented; L0209 and L0522 are implemented and not
  in the spec.
- **No types in lints.** `floats.rs` guesses "is a float" from the source,
  `fields.rs` matches fields by name; both are noted in their module docs.
- **One file per family.** `rules/taste/mod.rs` is still 1,890 lines of
  independent sequence matchers; a new idiom family gets its own file under
  `rules/taste/`.
- **Codemods in the linter.** `cpl fix`'s removed-syntax rewrites live in
  `src/lib.rs`, not in a migration module.
- **Fmt cannot reorder.** Sorted `use` lines and the line-1 stamp
  (`spec/tools/fmt-lint.md`) are not implemented.
