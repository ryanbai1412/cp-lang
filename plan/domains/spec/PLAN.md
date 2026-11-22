# spec plan (detail behind README.md)

Dated 2026-11-22. Numbers cite `metrics.toml`.

## Workstreams, in order

1. **Land what exists** (hours). Rebase `devin/wip-conformance-matrix` (6 commits, `git cherry` shows all unlanded) on main, regenerate `COVERAGE.md`, gate, land. Adds the matrix generator (`scripts/conformance-matrix.py`, `spec/conformance/matrix.toml`, 211 cases) and 108 hand cases with rulings 211-215.
2. **Rule completeness (M1).** For each of the 45 happy-path-only rules write the smallest reject case; fix the 78 unknown cites (teach `--rules` that `std.*` ids are std items, via a request to whoever owns `cpl-conform`; fix malformed headers myself); give the 10 rule-less core cases a rule. Then add `conformance-coverage.py --check` to the fast gate (request to infra).
3. **Trait table (M2).** `scripts/conformance-traits.py`: parse `pub trait` bodies in `spec/std/*.cp` and the core traits in `spec/core/08` and `12`; parse `impl Trait for Type:` claims (72); core claimers (List, Str, `[D]T`, Range, domains) by hand in `spec/conformance/traits.toml`. Each (type, trait, method) needs a snippet + expected output or an entry in `trait-gaps.txt`, which only shrinks.
4. **Spec blocks (M2).** A `<!-- given: ... -->` prelude before a fence supplies the names a fragment uses; a `<!-- spec-only: reason -->` marker skips it; tag the 27 untagged blocks. The `spec_blocks.rs` change is a request to the runner's owner (patch drafted on the wip branch under `research/conformance-generators/`).
5. **Consistency (M3).** A check that every std signature named in a chapter matches `spec/std` (names, named args); resolve rebuild round 2's 32 holes (ruling, case, or question); then rebuild round 3.
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
