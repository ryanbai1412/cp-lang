# Quality

| | |
|---|---|
| Kind | Process |
| Owner | tech-debt |
| Status | Active |
| Describes | `handbook/domains/*.md` (Quality register), `scripts/debt-ratchet.py`, `scripts/check-handbook.py` |

How we make sure CP Lang is good, and know it: the mechanisms we use, the
quality register each domain keeps, which test to write for which change,
how we measure whether a suite is any good, and what happens when a bug
gets past it.

## Goals

- Every quality attribute a user could feel has a check that runs without
  anyone remembering to run it.
- A bug class, once found, can't come back.
- We can tell a strong suite from a weak one by a number, not by its size.

## The toolkit, strongest first

Use the strongest mechanism that can decide the question.

1. **Make the bug impossible.** Re-architect so the wrong code can't be
   written. One engine can't disagree with itself; one entry point can't
   drift from a second one.
2. **Generate the tests.** Generated suites, fuzzers, property tests,
   differential tests (interpreter vs C++, CLI vs editor) and metamorphic
   tests (renaming or reformatting leaves output unchanged) cover inputs
   nobody thought of.
3. **Write the tests.** Conformance cases cited to a spec rule, snapshot
   suites, regression tests.
4. **Lint, ratchet or gate.** Count the bad pattern and let the count only
   go down.
5. **Process.** Review, cold probes, drills. Only where no mechanism can
   decide, and named as such.

For example, interpreter and C++ disagreeing on how a `HashMap` prints is a
level-2 find (a generated differential variant). Its fix is level 1: one
display rule per type that both back ends use.

## Quality registers

Each domain charter in `handbook/domains/` has a *Quality register*: every
quality attribute the domain must be high on, found by walking every
journey in `handbook/product/journeys.md` and every feature the domain
owns, and by asking how mature projects measure the same thing.

| Attribute | Metric | Mechanism | Check |
|---|---|---|---|
| Interpreter and C++ agree | Programs whose outputs differ | Differential runs on generated programs | `crates/cpl-cli/tests/fuzz_three_way.rs` |
| Every builtin agrees at its edges | Builtins with no edge case | Generated differential cases | `missing` |

- The cells after the metric name a real test file, script, command or CI
  job, or say `missing`, or start with `process` where nothing mechanical
  can decide. A final status column (`enforced`, `partial`, `missing`) is
  allowed; a row whose status says `missing` counts as missing.
- Targets and current numbers live in `plan/domains/<domain>/metrics.toml`,
  never in the charter.
- `python3 scripts/debt-ratchet.py --list register_rows` lists every row
  across all domains that names no existing check.
- Build `missing` rows in order of user impact across the whole register.

## Which test to write

| Change | Write |
|---|---|
| A language rule | A conformance case citing the rule, with a reject case if it forbids something |
| A bug fix | The program that exposed it, as a regression test in the suite whose row should have caught it |
| A std function | Doc examples that run, plus an interpreter-vs-C++ case at its edges |
| A diagnostic | A golden diagnostic case, and its fix applied by `cpl fix` |
| A lint | Firing and non-firing cases, and its fix |
| An editor feature | An LSP fixture, plus a parity case against the CLI |
| A new entry point or client | A parity case in `crates/cpl-wasm/tests/parity.rs` or `crates/cpl-cli/tests/lsp_parity.rs` |
| A trace or debugger change | A saved trace recorded on both engines |
| A performance change | A matched-algorithm benchmark; behavior never changes |
| A refactor | Nothing new if the suites already cover it; if the mutation score of the touched code is low, raise it first |

## How suites are measured

- **Mutation score.** Change the code in small ways (flip a comparison,
  drop a statement) and count how many changes some test catches. A suite
  that passes on broken code isn't checking it. `cargo-mutants` runs on
  `cpl-check` and `cpl-lint` first.
- **Coverage matrices.** For generated suites, the cells (feature by
  context, type by consumer) that have no case.
- **Escapes.** Bugs found by users or other domains that the suite should
  have caught.

## Rules live in guidelines

A rule is written once, as an item in a handbook guideline or checklist,
with its reason and an example. Everything that enforces it points there.

For example, "a search that can find nothing returns `?T`" is an item in
the std API guideline. The lint that flags a `-1` sentinel return cites
`handbook/guidelines/std-api.md#return-conventions` in its record, the test
that checks every std search names the same heading, and the stdlib
register row's Mechanism cell names the lint and the heading. Nobody restates
the rule.

- **One home per rule.** A rule stated only in a register row, a lint, a
  script or a worklog moves into the guideline that owns that kind of work:
  std API, std docs, errors, lints, performance, testing or writing. If that
  guideline doesn't exist yet, create it.
- **General items, not one-off rules.** Several narrow rules are usually
  one item stated generally ("absence is a type, never a sentinel" covers
  searches, lookups and parsers). Prefer that item to a list of cases.
- **Enforcers cite the item.** A lint's record (`spec/lints/<name>/`), a
  gate script's opening docstring, or a test's name or doc comment carries
  `handbook/<doc>.md#<heading>`. A lint is how a guideline is enforced.
- **Measured.** `python3 scripts/debt-ratchet.py --list uncited_rules`
  lists lints and gates that cite no existing heading. The count is
  reported now and becomes a gate once domains have folded their rules.

## Accountability

- **Every row names a real check.** Empty rows are counted per domain and
  go down.
- **Every lint and gate cites its rule.** See
  [Rules live in guidelines](#rules-live-in-guidelines).
- **Skips only shrink.** Ignored tests, xfails, allowlists and lists of
  known differences have shrink-only counts.
- **Expected output changes for a reason.** A commit that edits expected
  output together with the code that produces it is flagged for post-land
  review: it may have blessed a bug.
- **Escapes fix the suite.** Each escaped bug is logged in
  `plan/escapes.toml` with the domain and register row that should have
  caught it. That domain fixes the suite as well as the bug, and records the
  commit as `suite_fix`. `python3 scripts/debt-ratchet.py --list
  escapes_open` lists the entries still open.
- **Process costs under 10% of your time.** Above that, skip it and write
  `Skipped: <process>, <reason>` in the commit. The post-land review picks
  it up.

## Checklist

- [ ] The change has a test from the table above.
- [ ] No test was weakened, skipped or re-blessed to make it pass.
- [ ] If a bug escaped, the escape is logged and the suite fixed.
- [ ] Any new attribute the change creates has a register row.

## Changes

- 2026-11-23: first draft.
