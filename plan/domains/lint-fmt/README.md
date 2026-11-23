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

## Milestone 1 (plan/product/MILESTONE-1.md; due ~09:30 PST 2026-11-23)
Lint is passive teaching (J2, J5): a 1500+ newcomer's C++-shaped code gets the
CPL spelling, in the editor (Neovim/Vim first, VS Code too) and in the browser
playground. Format on save just works on Linux, macOS and Windows. Lint and
fmt don't touch the generated C++, so the Codeforces profile needs nothing
from this domain. Checkpoints every 2 h in `worklog.md`.

**Exit criteria**
1. Playground: each record's `before.cp` warns, with its fix, exactly as
   `cpl lint` does. Wasm side landed (`Playground::lint`, test
   `the_playground_lints_like_cpl_lint`); the editor calling it is site's.
2. Format on save, no setup: LSP formatting follows `cp.toml [fmt]` (landed);
   VS Code defaults it on (landed); the Neovim plugin formats on write
   through the LSP, and Vim gets a documented `cpl fmt` hook; both smoke
   tests cover it.
3. Windows: `cpl fmt` keeps a CRLF file CRLF, so `cpl fmt --check` passes on
   a `core.autocrlf` checkout (test in `cpl-fmt`).
4. Newcomer ranking: lints ranked by hits on the 1,160 corpus `port.cp` files
   (C++ translations, the nearest proxy for newcomer code; today L0414 279,
   L0504 57, L0503 42, L0523 37, L0507 36, L0524 33), kept in
   `metrics.toml`. Each of the top 10 names the CPL spelling, has a verified
   fix where meaning allows and an explain page; 0 false positives on corpus
   `spec.cp` and `std`.
5. The top-ranked lint is worth seeing: L0414 (279 port hits) is precise
   enough to teach, or leaves the default set.

**Deferred** (milestone "can wait", or not newcomer-facing; continues only
after the above): M3 recognizers and library `@lint`, moving `bounds.rs` onto
the fact table, sorted `use` and the line-1 stamp, catalog families with no
port evidence, lint speed work.

**Next 3** (in hours; status 2026-11-23 00:20 PST: newcomer 147/151, 97%)
1. By 03:00: the 3 newcomer misses that need type or flow facts land with
   diagnostics (int ids → domain ×2, found flag beside a value → `?T`).
2. By 06:00: audit the top 10 port lints' messages and fixes, highest count
   first, against `plan/domains/idioms/` principles.
3. Until the deadline: more teaching lints by port and newcomer evidence,
   each with 0 port/std false positives and its near-miss shapes.

## Milestone 2 (plan/product/MILESTONE-2.md; stable release, no date)

**Most ambitious complete version.** Lint is the teacher a newcomer never
needs: every C++ habit and every stdlib idiom in `spec/idioms/lints.md` has a
rule with a record (before, after, near miss, why), each finding links its
page and carries a fix that `cpl check` accepts, rules read the checker's fact
table instead of syntax guesses, and libraries declare their own lints with
`@lint`. `cpl fmt` is the one layout, configurable through `cp.toml [fmt]`,
idempotent and meaning-preserving on every file, and on save in every
supported editor on Linux, macOS and Windows.

**Exit criteria**
- At least 95% of marked spots flagged on the newcomer set and on the skill-eval
  first tries, with 0 fixes that fail `cpl check`.
- Every catalog family with port or newcomer evidence has a rule; the rest
  are logged as deferred, each with its reason.
- Corpus and `spec/std` warnings at 0, or each one a recorded real finding.
- Fact-table lints only: no rule re-derives a type from syntax.
- Library `@lint` declarations work for user libraries the same way as for std.
- The fmt gate passes on the whole corpus, and format on save is tested in
  VS Code, Neovim, Vim and JetBrains on all three platforms.

**Next steps**
1. Finish M1's 90% newcomer target, then score the skill-eval outputs.
2. Move `bounds.rs` and the typed arms onto the fact table.
3. Library `@lint` (M3 recognizers).

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
   tracked `.cp` files; `cpl fmt --check` clean on corpus ports, `std` and
   docs snippets, enforced in `scripts/check.sh --fast`.
2. **Lint has no false positives where it matters.** 0 warnings on
   `corpus/problems/*/spec.cp` and `std` at default levels, every
   remaining hit classified (false positive fixed, or port fixed by corpus).
3. **Every lint is a record.** Each lint has `before/after/near_miss` tests;
   every `fix` is verified (re-checks; interpreter and C++ agree).
4. **No privilege.** The linter names no stdlib API; libraries declare lints
   with `@lint` (Ryan: yes), and user libraries can too.
5. **One engine everywhere.** CLI, LSP and playground give the same findings
   (today the browser loses the checker-backed lints).
6. **Fmt "just works"** (Ryan's target): it is configurable, and its defaults
   are very opinionated. It runs on save with nothing set, and a `cp.toml
   [fmt]` table changes the few layout choices a project may want.
7. **Lint is highly configurable** (Ryan's target): it is easy to turn off a
   rule you don't like. A lint, or a whole group of them, is turned off with
   one `cp.toml [lints]` line or one `with lint(name: allow)`.

## Milestones
**M1 Fmt gate on main (first; days).** Land `devin/wip-fmt`; fix the one
meaning change; one mechanical commit formats the owned trees.
Exit: `python3 scripts/fmt_check.py` reports 0 not idempotent, 0 meaning
changes, 0 bugs; `cpl fmt --check corpus/problems/*/spec.cp std/*.cp`
says 0 would change; `scripts/check.sh --fast` runs the fmt step.

**M2 Lint as data, corpus-clean (the rebuild's spine).** One record per lint
under `spec/lints/<name>/`, Rust registry generated from it; the 38 lints gain
near-miss and fix tests; the false-positive ratchet runs in the gate.
Exit: records = `cpl lint --list | wc -l`; `cpl lint corpus/problems/*/spec.cp`
and `std/*.cp` report 0 warnings; a test fails when a record and the
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
| Warnings on corpus `spec.cp` (100) | `cpl lint corpus/problems/*/spec.cp` | 34, 1 not linted | 0, 0 |
| Warnings on `std` (40) | `cpl lint std/*.cp` | 262, 5 not linted | 0, 0 |
| Warnings on `dream.cp` (421) | `cpl lint corpus/problems/*/dream.cp` | 99, 84 not linted | 0 false positives |
| Hard-coded stdlib names in rules | `grep -ohE '"[A-Z][A-Za-z]+"' rules/*.rs` | 16 | 0 |
| Wasm-only gaps in cpl-lint | `git grep -c 'target_arch = "wasm32"'` | 4 (all the prover) | 4 (decisions.md 235) |
| Fixed files with more checker errors (std + corpus spec) | test `fixes_never_add_checker_errors` | 0 | 0 |
| Lint time, largest linted file (2,682 lines) | `time cpl lint std/tree.cp` | 0.01 s | < 0.05 s |
| Fmt time, same file | `time cpl fmt --check std/tree.cp` | 0.00 s | < 0.05 s |
| `.cp` files fmt would change | `python3 scripts/fmt_check.py` | 585 / 5,176 | 0 outside exempt tests |
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
- **Lint and fmt.** Ryan set the targets in success criteria 6 and 7 and sees a
  demo on real ports. The calls that were with him are decided (see
  "Decided"), and he can reverse any of them.

## Decided
The three questions this packet put to Ryan were decided by their
recommendations (Ryan's rule: an owner makes reversible calls):
- severity groups and the v0.3 lint set: decisions.md 275;
- what `cpl fmt` formats, and where the stamp goes: decisions.md 276;
- the design: decisions.md 277.

Nothing in lint-fmt is waiting on Ryan.
