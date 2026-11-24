# spec plan (detail behind README.md)

Dated 2026-11-22. Numbers cite `metrics.toml`.

## Milestone 1 order (2026-11-22, `handbook/product/milestones.md`)

Milestone 1 sets the order; the workstreams below are means to it.

1. **Freeze the subset.** `spec/SUBSET.md`: per chapter section and lib family, *supported* (cases pass), *documented*, or *known gap* with its owner line. Built from `cpl-conform conformance` per chapter; what fails in a *supported* row is routed or the row moves to *known gap*, never hidden.
2. **Finish range keys** (decision 289): audit lib chapters and std docs; 3 of 7 cases fail on the compiler and stay visible.
3. **Examples in the subset run** (workstream 4 narrowed to supported chapters).
4. **Writable from the docs** (workstream 5): rebuild rounds on Div. 2 A-D problems; contradictions inside the subset first.
5. **Journey tool contracts**: rule ids and cases for `check`, `run`, `test`, `stress`, `build` (cli, fmt, lint, bundle, stress landed).
6. Deferred to after the milestone: trait/matrix cells outside the subset, trace/DAP/explain/LSP rule ids, more bug-catch rounds, R11.4.3.

Workstreams 1-3 below are landed; 4-6 continue in the order above.

## Workstreams, in order

1. **Land what exists** (hours). Rebase `devin/wip-conformance-matrix` (6 commits, `git cherry` shows all unlanded) on main, regenerate `COVERAGE.md`, gate, land. Adds the matrix generator (`scripts/conformance-matrix.py`, `spec/conformance/matrix.toml`, 211 cases) and 108 hand cases with rulings 211-215.
2. **Rule completeness (M1).** For each of the 45 happy-path-only rules write the smallest reject case; fix the 78 unknown cites (teach `--rules` that `std.*` ids are std items, via a request to whoever owns `cpl-conform`; fix malformed headers myself); give the 10 rule-less core cases a rule. Then add `conformance-coverage.py --check` to the fast gate (request to infra).
3. **Trait table (M2).** `scripts/spec-gate.py traits`: parse `pub trait` bodies in `std/*.cp` and the core traits in `spec/core/08` and `12`; parse `impl Trait for Type:` claims (72); core claimers (List, Str, `[D]T`, Range, domains) by hand in `spec/conformance/traits.toml`. Each (type, trait, method) needs a snippet + expected output; the gate fails on one without.
4. **Spec blocks (M2).** A `<!-- given: ... -->` prelude before a fence supplies the names a fragment uses; a `<!-- spec-only: reason -->` marker skips it; tag the 27 untagged blocks. The `spec_blocks.rs` change is a request to the runner's owner (patch drafted on the wip branch under `research/conformance-generators/`).
5. **Consistency (M3).** A check that every std signature named in a chapter matches `std` (names, named args); resolve rebuild round 2's 32 holes (ruling, case, or question); then rebuild round 3.
6. **Probes (M3).** Script P1 (rebuild) and P2 (revert past `crates/` fix commits, run the suite, count reds) so the coordinator can rerun them.

## Order and why

- **M1 first** because a rule with only accept cases cannot catch the bug that matters most (the compiler accepting what it should reject), and it is pure spec work, unblocked.
- **M2** because handoff evidence says most recent compiler bugs were combinations (index types x consumers: 46 of 121 matrix cells failed), which hand-written per-rule cases miss.
- **M3** because coverage counts can be gamed; only a cold rebuild and reverted-fix catch rate show the spec says enough and the cases bite.

## Alternatives considered

- *Mutation testing of the compiler* instead of P2: more thorough but slow on 2 cores and noisy; reverting real past fixes samples real bug shapes. Revisit if P2 saturates.
- *Write cases by hand for traits*: no completeness signal; a generator plus a shrink-only gap file makes "every method on every claimer" checkable.
- *Delete failing spec blocks*: loses examples; the `given`/`spec-only` markers keep them honest.
- *Spec owning the runner crate*: proposed to the coordinator (README); until then runner changes go as requests.

## Risks

- Pass rate drops as coverage grows (new failing cases). That is the point; report cases added and cases passing separately.
- Parallel lanes regenerate `COVERAGE.md`; conflicts are resolved by regenerating, never by hand.
- Rulings that need Ryan (QUESTIONS.md) could leave cases contradicting the spec; meanwhile the case follows the stated default and cites the question.
- Ownership gap on `crates/cpl-conform` slows M1 and M2 tooling.

## First steps in phase 2

1. Rebase and land `devin/wip-conformance-matrix`.
2. Reject cases for the R03, R09 and R13 happy-path-only rules (19 of the 45).
3. Fix malformed `// rule:` lines; file the `std.*` cite request.

## General fixes owed

Each local fix, and the mechanism that would remove its whole class.

| Instance | General fix | Owner |
|---|---|---|
| Rebuild 7: `std/README.md` lists `bsearch_first`, copying `windows`, `first(p)` that `std/*.cp` and spec/core do not have | generate the API listing in `std/README.md` from the `std/*.cp` declarations and doc comments, or a check that every signature it lists exists there | stdlib (spec adds the check to `conformance-signatures.py` if stdlib keeps hand-written listings) |
| Rebuild 7: `Str.count(c: Char)` beside `Seq.count(p)` | a check that no two `pub fn`s of one name apply to one receiver type (R06.1.6) over all of `std/` | compiler (as an error), stdlib (cleanup) |
| Rebuild 7: `Str(len:)` is E0504 inside a supported subset row | `conformance-subset.py` fails a supported row whose rule text names a form the compiler reports as E0504 | spec |
| Rebuild 7: answered rules guessed anyway (lambda tuple unpack R06.1.4, range precedence R01.5, blank lines R10.2.8) | each core chapter overview lists the rules a newcomer guesses, generated from rebuild HOLES files | spec |
| Rebuild 6-7: a spec fact and a std doc fact that disagree (`tally` type, `desc` flag) | `conformance-signatures.py` compares spec tables' result types with `std/*.cp` signatures for every row that names one | spec |
| `range_sum` compiler intrinsic for `(a..b).step(k).sum()` | folds are overridable trait defaults; the type's impl gives its cost (decision 486) | compiler (traits in std, delete intrinsic), stdlib (range overrides) |
| inherent `m` beside an implemented trait's `m` (R08.1.4, `core-08/inherent-before-trait.cp`): concrete and generic calls differ | lint an inherent method that shares a name with a method of a trait the type implements, suggesting the override | lint-fmt |
| `core-07/range-folds-are-ranges-own.cp` hung `cpl-conform` for 19 min: the in-process interpreter ignored `CPL_CONFORM_CASE_TIMEOUT` | `runner::run_interp_bounded` bounds conformance runs; next, `corpus.rs`, `docs.rs` and `diff.rs` use it too. A timed-out run is abandoned, so the real fix is interpreter fuel or an interrupt flag | spec (callers), compiler/interp (interrupt) |
| `core-09/deep-recursion-trap.cp` (recursion 1e8 deep, R0601) takes 7 s alone and passes, but times out at 20 s in a loaded full run, so `conformance-subset.py --check` flips red on a supported row | A depth limit or fuel counter in the interpreter so a stack-overflow trap costs O(limit) work, independent of host speed; meanwhile the case keeps its input | compiler (interp) |
| `(a..b).step(k).len` and `.min()` allocate GBs as they walk | a check that no view adapter allocates per element (R07.2.8), as a perf probe over every adapter | stdlib |
| `r.sum` without `()` gives no L0608 warning, because the lint looks only at builtins | L0608 checks the resolved method: any non-`@get` method read without `()`, std and user methods included | compiler |
| Round 7 could not tell whether Heap, Graph and Prefix are inside Milestone 1 (H-LIB-SUBSET-STATUS) | every `conformance/std-*` folder is a `SUBSET.md` row that `conformance-subset.py --check` enforces like a rule section; a new family can't be missing from the subset | spec (done) |
| A bug-catch tree old enough to fall back to its own cpl-conform ran without case timeouts and hung for the 15 min suite limit | measure every tree with HEAD's runner driving that tree's `cpl`, so harness fixes reach old trees | spec (done) |
| Bug-catch round 9: two codegen fixes (`718cce42`, `d80f7a08`) are invisible because cpl-conform runs only the interpreter | `CPL_CONFORM_RELEASE=1` runs cases through `cpl run --release`; `spec-gate.py bugcatch --release` uses it (too slow per push, so for bug-catch rounds) | spec (done) |
| Bug-catch counts perf-only codegen fixes (`718cce42`) as misses, but no output case can see them | tag a fix `cost-only` in bugcatch and measure it with perf's bench instead (an op-count or time budget per case) | perf (bench), spec (tagging) |
| Lone-CR reject cases put `at 4` (file line) where `at N` counts from after the directive headers, and a plain comment line shifts it; landed red twice | Report a wrong line as `at N (file line M)` in cpl-conform's failure detail, and make the land path fail on `conformance-subset.py --check` instead of piping it through `tail` | spec |
- Instance: std fields used by spec cases stayed private under decision 365 and broke when E0207 landed. General fix: the `pub` marking counts `spec/lib/` and `spec/conformance/` uses as outside the module, and the compiler lands a visibility check only after the conformance suite passes with it. Owner: stdlib (marking), compiler (ordering).
- Instance: every rebase of a spec commit conflicts on the generated `spec/conformance/COVERAGE.md` and is fixed by hand (`--theirs`, regenerate). General fix: a `merge=regen` driver for every `gen.toml` file, so a rebase takes either side and land.sh's regen step rewrites it. Owner: infra (gen.toml, .gitattributes, land.sh).
- The v0.2 CLI program suite (438 files at `6f487909a~1:crates/cpl-cli/tests/programs/`) was never classified against the spec. General fix: run it through `cpl-conform`, turn each program that tests a rule with no case into a conformance case, and drop the rest. Owner: spec.

## v0.3 success criteria and milestones

1. Every rule that forbids, traps or limits has a reject case; every rule has an accept case unless it only forbids. (2 forbidding rules lack a reject case today; 3 rule gaps in all; 26 rules that only permit or define are listed with a reason in `spec/conformance/permits.txt`.)
2. Every case cites a rule, a std item or a diagnostic code that exists. (0 bad cites, 0 rule-less core cases today.)
3. The feature x consumer matrix and the trait x claimer x method table have no empty cell (case or written n/a reason).
4. Every spec code block is a case or is marked spec-only. (19 of 185 tagged blocks pass, most fail as fragments naming undefined values; 27 untagged blocks are skipped.)
5. A cold agent rebuilding from `spec/` alone finds fewer than 5 holes. (Round 2 found 32.)
6. All of the above are ratchets in `scripts/check.sh --fast`.


- **M1 Every rule tested both ways.** Exit: `conformance-coverage.py` reports 0 happy-path-only rules, 0 unknown cites, 0 rule-less core cases, 0 diagnostic codes without a reject case; `conformance-coverage.py --check` runs in the fast gate.
- **M2 Generated coverage.** Exit: `devin/wip-conformance-matrix` landed and `spec-gate.py matrix` gates 0 empty cells; `spec-gate.py traits` gates the trait table, failing on any method with no case; spec-blocks: every fence is a case or `spec-only`.
- **M3 Independently rebuildable.** Exit: a rebuild round (cold agent, `spec/` only, 40 questions + 10 programs) records < 5 holes; the bug-catch probe catches >= 90% of reverted compiler fixes.
