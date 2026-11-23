# lint-fmt plan

The detail behind [README.md](README.md). Target design:
[design/overview.md](design/overview.md). Numbers: [metrics.toml](metrics.toml).

## Workstreams, in order
1. **Fmt gate (M1).** Land `devin/wip-fmt` (4 commits: c-spelling spacing,
   `impl`/`lends`/`without`/`as` spacing, leading-dot shorthands, `[n]T`,
   member keywords; `scripts/fmt_check.py` in the fast gate; a design note on
   line breaking). Fix the `c_operators.cp` meaning change with a regression
   case in `crates/cpl-fmt/src/tests.rs`. Then one mechanical commit formats
   corpus ports, `spec/std`, examples and docs (layout tests exempt, per the
   question in `plan/QUESTIONS.md`).
2. **Lint as data (M2).** Land `devin/wip-idioms-lint`'s 38 `spec/lints/<name>/`
   records; generate `LINTS` from them (build script or a checked-in generated
   file with a drift test); add a test that the records and the lint table in
   `spec/diagnostics.md` agree. Behaviour unchanged.
3. **False-positive ratchet (M2).** Classify each of the 50 corpus `spec.cp`
   and 293 `spec/std` warnings: a false positive becomes a near-miss case and a
   fix in the lint; a true finding becomes a request to corpus or stdlib. Gate
   the count in `scripts/check.sh --fast` so it only goes down.
4. **Facts from the checker (M3, needs compiler).** Contract tests in
   `crates/cpl-lint` for the facts the lints need; the compiler exposes a fact
   table; the linter and `cpl-lsp` read it; delete `rules/bounds.rs` and the
   `Proven*` passes. Until then the current analysis stays, unchanged.
5. **Recognizers and `@lint` (M3).** fold, search, build, scan first (they
   cover `rules/taste/`'s largest lints); port `hand_rolled_*`; stdlib `@lint`
   declarations replace the hard-coded names; then list L0601-L0609, which
   `cpl-check` already emits, in the registry.
6. **Spelling cleanup.** `@allow` removed in favour of `with lint(...)`
   (`cpl fix` rewrites it), after the spec domain confirms.
7. **Grow the catalogue.** From the idioms catalog, P0 and bound-aware
   correctness first, each as a record plus `@lint`, opt-in until its corpus
   run has 0 false positives.
8. **Fmt reordering.** Sorted `use` runs and the line-1 stamp, each a pass
   with its own verify step. The handoff's E0206 cascade (sorting changed
   diagnostics in the old `examples/local-lib`) does not reproduce on main's
   copy, `research/archive/local-lib-v02/spec/worked.cp`: 1 E0206 before and
   after sorting; `fmt_check.py` re-checks meaning anyway.

## Why each milestone
- **M1 first** because fmt is the smallest, nearly done, and every later
  bulk change (fixes, codemods, ports) is printed by it; a fmt that can change
  meaning makes every autofix suspect.
- **M2 before new lints** because 343 warnings on idiomatic code mean the
  existing lints are not trustworthy yet, and records make each lint's
  contract (fires / silent / fix) a test instead of prose.
- **M3** is the architecture: it removes the duplicate analysis and the
  stdlib privilege, and is what makes ~670 catalogue entries cheap. It depends
  on the compiler lane, so it runs in parallel with M2's ratchet.

## Alternatives considered
- **Fmt on a parsed tree (rewrite as a pretty-printer).** Would allow
  reordering and smarter breaks, but loses formatting of broken files and
  needs a full lossless tree; the token approach already holds 0
  non-idempotent files. Kept tokens; reordering gets its own verified step.
- **Keep lints in Rust, add records only as tests.** Less churn, but the
  registry, the spec table and the docs keep drifting (already 11 codes
  disagree). Rejected.
- **Linter keeps its own analysis.** Avoids a compiler dependency, but keeps
  ~3k duplicated lines and two answers to "is this index proven". Rejected.
- **All idiom lints default-on now.** Rejected by Ryan's corpus-gate rule.

## Risks
- The fact table slips in the compiler lane: M3 waits; M1, M2 do not.
- The bulk format collides with other lanes' in-flight edits to ports: land it
  as one mechanical commit at a quiet moment, announced in the corpus worklog.
- The ratchet finds that many warnings are true: then the fix is in the ports
  (corpus) or stdlib, not in lint; tracked as requests, not suppressed.
- Generated registry adds build complexity: prefer a checked-in generated file
  plus a drift test over a build script.
- `devin/idioms-docs` and `devin/wip-idioms-lint` overlap (shared commits):
  `git cherry` before landing, so nothing lands twice.

## Phase 2, first
1. Rebase `devin/wip-fmt` on main, renumber its decision if 214 is taken,
   run `scripts/check.sh --fast`, land with `CPL_LANE=lint-fmt scripts/land.sh`.
2. Regression case and fix for the `c_operators.cp` meaning change.
3. `git cherry` `devin/idioms-docs` against `devin/wip-idioms-lint`; land the
   lint records and engine part; hand `spec/idioms/*` to the idioms lane;
   move `plan/LINTER.md` to `design/linter.md`.
4. Start the ratchet: classify the 50 corpus `spec.cp` warnings.
