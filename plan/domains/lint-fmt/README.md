# lint-fmt

Segment: C (design, then build). Owns: `crates/cpl-lint`, `crates/cpl-fmt`.
Docs: [design/overview.md](design/overview.md) (what v0.3 lint and fmt are; the
main review item), [PLAN.md](PLAN.md), [ARCHITECTURE.md](ARCHITECTURE.md),
[metrics.toml](metrics.toml), [worklog.md](worklog.md). Normative rules:
`spec/tools/fmt-lint.md`, lint codes in `spec/diagnostics.md`.

## Mission
The source-to-source tools. `cpl fmt` gives every file one canonical layout and
never changes meaning. `cpl lint` points at legal code that is probably wrong,
slow or unidiomatic, says why in source terms, and offers only fixes that are
verified to check and to print the same output. Both are gated on the
idiomatic corpus, so a finding there is a bug in the lint, not the port.

## Covers
- **Lint.** The engine (levels, `with lint(...)`, `cp.toml [lints]`, fix
  application), every `L0xxx` lint, its record and tests, `cpl lint`,
  `cpl fix` codemods for removed syntax, the bound-aware "proven" analyses until
  the checker owns them.
- **Fmt.** The formatter, its style, `cpl fmt` / `--check`, the corpus-wide
  idempotence and meaning gate.

## Not covered (owner)
| What | Owner |
|---|---|
| Which idioms exist, their prose catalog (`spec/idioms/`) | idioms |
| Parser, lossless tree, checker facts, diagnostics API, wasm build of the checker | compiler |
| Lint codes' normative table, `spec/tools/fmt-lint.md` wording | spec |
| How lint/fmt surface in the editor (code actions, format on save) | tools (LSP) |
| Rendering of diagnostics, `cpl explain` text style | diagnostics |
| Ports that a lint shows are not idiomatic | corpus |
| Per-lint site pages, playground | site |

## Success criteria for v0.3
1. **Fmt is safe on everything.** Idempotent and meaning-preserving on all
   tracked `.cp` files; `cpl fmt --check` clean on corpus ports, `spec/std` and
   docs snippets, enforced in `scripts/check.sh --fast`.
2. **Lint has no false positives where it matters.** 0 warnings on
   `corpus/problems/*/spec.cp` and `spec/std` at default levels, every
   remaining hit classified (false positive fixed, or port fixed by corpus).
3. **Every lint is a record.** Each lint has `before/after/near_miss` tests;
   every `fix` is verified (re-checks; interpreter and C++ agree).
4. **No privilege.** The linter names no stdlib API; libraries declare lints
   with `@lint` (Ryan: yes), and user libraries can too.
5. **One engine everywhere.** CLI, LSP and playground give the same findings
   (today the browser loses the checker-backed lints).

## Milestones
**M1 Fmt gate on main (first; days).** Land `devin/wip-fmt`; fix the one
meaning change; one mechanical commit formats the owned trees.
Exit: `python3 scripts/fmt_check.py` reports 0 not idempotent, 0 meaning
changes, 0 bugs; `cpl fmt --check corpus/problems/*/spec.cp spec/std/*.cp`
says 0 would change; `scripts/check.sh --fast` runs the fmt step.

**M2 Lint as data, corpus-clean (the rebuild's spine).** One record per lint
under `spec/lints/<name>/`, Rust registry generated from it; the 38 lints gain
near-miss and fix tests; the false-positive ratchet runs in the gate.
Exit: records = `cpl lint --list | wc -l`; `cpl lint corpus/problems/*/spec.cp`
and `spec/std/*.cp` report 0 warnings; a test fails when a record and the
registry or `spec/diagnostics.md` disagree.

**M3 Facts from the checker, lints from libraries.** Bound facts come from a
`cpl-check` fact table (compiler); shape recognizers; `@lint` in the stdlib;
hard-coded names gone; L0601-L0609 (emitted by `cpl-check` today) listed and
levelled by `cpl lint`.
Exit: 0 `"Int"`-style stdlib names in `crates/cpl-lint/src/rules/`; 0 wasm
gates in `crates/cpl-lint`; a user-library `@lint` test fires and fixes;
`cpl lint --list` shows every `L` code in `spec/diagnostics.md`.

## Metrics
Measured 2026-11-22 on main (98618e1b5) with a release `cpl`; commands in full in metrics.toml.

| Metric | Command (abridged) | Today | Target |
|---|---|---|---|
| Lints implemented | `cpl lint --list \| wc -l` | 38 | 38 + catalog adds (L0601-L0609 are checker-emitted) |
| Lints with records + tests | `ls -d spec/lints/*/ \| wc -l` | 38 | = lints |
| Spec lint codes emitted by nothing | `comm` of `--list` vs `spec/diagnostics.md`, then grep `cpl-check` | 0 (9 in `cpl-check`) | 0 |
| Implemented codes missing from spec | test `every_lint_matches_the_spec_lint_list` | 0 | 0 |
| Warnings on corpus `spec.cp` (100) | `cpl lint corpus/problems/*/spec.cp` | 43, 1 not linted | 0, 0 |
| Warnings on `spec/std` (40) | `cpl lint spec/std/*.cp` | 269, 5 not linted | 0, 0 |
| Warnings on `dream.cp` (421) | `cpl lint corpus/problems/*/dream.cp` | 80, 84 not linted | 0 false positives |
| Hard-coded stdlib names in rules | `grep -ohE '"[A-Z][A-Za-z]+"' rules/*.rs` | 16 | 0 |
| Wasm-only gaps in cpl-lint | `git grep -c 'target_arch = "wasm32"'` | 8 | 0 |
| Lint time, largest linted file (2,682 lines) | `time cpl lint spec/std/tree.cp` | 0.01 s | < 0.05 s |
| Fmt time, same file | `time cpl fmt --check spec/std/tree.cp` | 0.00 s | < 0.05 s |
| `.cp` files fmt would change | `python3 scripts/fmt_check.py` | 559 / 5,080 | 0 outside exempt tests |
| Not idempotent | same | 0 | 0 |
| Meaning changes | same | 0 | 0 |
| Corpus `spec.cp` not fmt-clean | `cpl fmt --check corpus/problems/*/spec.cp` | 28 / 100 | 0 |

## Judgment rubric
1. **Would a strong contestant thank the lint?** The message names the fact
   ("`I.sorted` copies; `I.sort` sorts in place"), the fix reads better than
   the original, and it never fires on code a reviewer calls idiomatic.
2. **Is the fix trustworthy without reading it?** Applied blind, it keeps
   meaning and layout; when unsure the lint is silent or offers no fix.
3. **Is fmt output what a careful person writes?** No surprising breaks, no
   churn on revision: a one-token edit reformats one chunk.
4. **Is a new lint cheap?** A library author adds one without Rust.
5. **Do the crates stay small and honest?** No duplicated checker logic, no
   per-lint special cases in the engine.

## Probes (a cold agent attempts, from docs only)
- Write a new lint for a stdlib API (`@lint` + record) and land its tests;
  measure files touched and whether any Rust was needed (target: none, M3).
- Take 10 random ports, mutate one into a known anti-pattern each; count lints
  that fire, fixes that apply, fixes that still pass the port's tests.
- Revise a formatted port (add a DP dimension); count lines `cpl fmt` moves
  that the edit did not touch (target: 0).
- Report a false positive and turn it into a near-miss case; time to green.
- Format every docs snippet and the site examples; any diff in meaning fails.

## Improve (highest impact first)
1. Fix fmt's one meaning change (`crates/cpl-cli/tests/fixes/c_operators.cp`:
   E0107 becomes E0102 after formatting) and land the fmt gate (M1).
2. Lint records + generated registry; ratchet corpus and std warnings to 0
   (M2). Today 343 warnings sit on code that is idiomatic by definition.
3. Move bound facts into `cpl-check`; delete `bounds.rs` and the "proven"
   passes (~3k lines; LSP and lint read one table) (M3).
4. Shape recognizers and `@lint`; delete hard-coded names; split `taste.rs`
   (4,493 lines) by recognizer.
5. Same lints in the browser (checker in wasm, compiler's call).
6. Sorted `use` lines and the line-1 stamp, each behind its own verify step.

## Ryan's role
- **Lint.** Reviews `design/overview.md`, the v0.3 lint list and the severity
  policy (error vs warning vs off); then a demo on real ports.
- **Fmt.** Picks the few style calls the spec leaves open, once.

## Decisions for Ryan
Each is also in `plan/QUESTIONS.md` with options and a reversible default.

1. **Severity policy and v0.3 lint scope.** Recommend: `correctness` deny
   (fails `cpl lint`, never `cpl build`), the rest warn after the corpus gate;
   v0.3 = the 38 as records + L0601-L0609 + P0 / bound-aware families.
2. **What fmt may touch.** Recommend: one mechanical commit formats corpus,
   `spec/std`, examples and docs; layout tests in `spec/conformance` and
   `crates/*/tests/fixes` are exempt; the stamp goes only on formatted files
   outside those.
3. **Approve the design** ([design/overview.md](design/overview.md)): facts
   from one checker table, lints as data records, fmt stays token-preserving.
   Recommend: approve; M1 and M2 hold under any answer.
