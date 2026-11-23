# diagnostics plan

Detail behind [README.md](README.md). Numbers: [metrics.toml](metrics.toml).

## Where we are
Main does not meet its own recovery bars: both scoreboards in
`crates/cpl-cli/tests/recovery.rs` are `#[ignore]`d (98.6% first-right,
8.9% cascaded, 59.2% fixable). The handoff branch `devin/wip-diag-recovery`
(6 commits, `git cherry` shows none landed) was measured at 99.5% /
1.1% / 97.2% when it was written; that was before main moved, so it is
not claimed here until rerun on the rebased tip.

## Workstreams, in order
Milestone 1 (`plan/product/MILESTONE-1.md`) orders the work: step 5,
"diagnose ordinary mistakes", for a newcomer bringing C++ habits. M1 of
the old list (gated recovery) has landed; M2/M3 numbers are means now.
1. **Newcomer mistakes (Milestone 1).** Mine the first-attempt failures
   in `research/dryrun/LOG-v03.md` and what a C++ contestant would type
   for each of the 23 practice problems in `web/content/learn/practice/`;
   check them in as a third scoreboard in `recovery.rs` whose bar is "the
   fix runs the sample". Fix its worst rows top-down, one commit each.
2. **No wrong fixes.** Any fix that leaves an error or changes meaning
   is removed or corrected before new fixes are added (first:
   `fn`/`struct` bodies in braces).
3. **Runtime traps.** Every R-code reachable from the practice problems
   names its values and the CPL fix; a `runtime_errors` case each.
4. **Editor parity.** LSP shows the same errors and quick fixes as the
   CLI (`lsp_parity`); requests to tools when it does not.
5. **Fix-its and messages (old M2), only where the newcomer set or the
   probes hit them.** Generic `expected ...` errors, catalogue examples.
6. **Root cause (old M3), deferred.** Benchmark base from every
   `spec.cp`, `cpl-ir` sink suppression, cascades <= 2%; done only when
   cheap or when the newcomer set shows a cascade.

Each workstream lands in small commits, each with the test or scoreboard
row that moved.

## Why these milestones
- M1 first because nothing gates recovery today; every other lane can
  regress it silently. It is also mostly landed work waiting.
- M2 is the contestant-visible win: a fix-it is one keystroke.
- M3 is the long tail; it needs a benchmark that grows with the corpus,
  or the numbers overfit 37 ports.

## Alternatives considered
- *Placeholder fix-its* (`let x = todo`) would lift broken-ports fix-it
  from 72% toward 90% at once. Rejected by default: a fix that compiles but
  is wrong breaks rubric Q2. Decision 283.
- *Stop after the first error per item* would zero cascades trivially but
  hide independent errors; the root-cause rule says one per cause, not one
  per item.
- *Lower the bars to un-ignore now* would gate a regression into place;
  never weaken a test to land.
- *Move error sites into `cpl-diag`*: no; the crate stays language-free and
  the rules stay with the checker. Diagnostics owns the wording and spans.

## Risks
- Error sites are in compiler-owned files; parallel compiler work causes
  conflicts. Mitigation: small commits, requests in the compiler worklog.
- Overfitting to six synthetic edits. Mitigation: the probes, growing
  benchmark base, spec-block scoreboard.
- Recovery changes cost front-end time. Guard: latency test (34 ms of 50).
- Branch measurements went stale while main moved (already happened once).

## Milestone 1, first actions
1. Write the newcomer cases from the dry-run log and practice problems
   under `crates/cpl-cli/tests/newcomer/`, add the scoreboard, measure,
   and gate it at the measured value.
2. Fix brace-bodied `fn` and `struct` (today a wrong fix), then the
   worst newcomer rows.
