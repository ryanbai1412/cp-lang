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
1. **Gate (M1).** Rebase and land `devin/wip-diag-recovery`; fix what
   remains under the bars (E0203 near-name fix-its, dropped-colon E0006
   cascade); delete the `#[ignore]`s so `check.sh` gates recovery.
   Replace the 11 raw `cpl_diag::Code("E…")` sites with `codes::`
   constants and update their *(0.3)* status (needs compiler: request in
   its worklog; the edits are mechanical).
2. **Fix-its (M2).** Walk the scoreboard's "first error by code" table
   top-down, adding a fix-it where one is right; raise the bars in
   `recovery.rs` as each lands. Fill the 33 empty catalogue examples from
   the registry triggers (the catalogue test then checks them).
3. **Messages (M2).** Replace generic `expected ...` first errors on spec
   blocks with rule-naming messages; draft the style guide from the rubric
   for Ryan's one sign-off.
4. **Root cause (M3).** Cascade work per edit row; make the benchmark base
   every `spec.cp` that checks clean (today a 37-entry list) so it grows
   with the corpus; give `cpl-ir` lowering the same `Sink` suppression.

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
  is wrong breaks rubric Q2. Filed in `plan/QUESTIONS.md`.
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

## Phase 2, first actions
1. `git rebase origin/main devin/wip-diag-recovery`, run
   `scripts/check.sh --fast` and the recovery scoreboards, land via
   `CPL_LANE=diagnostics scripts/land.sh`, record before/after in the
   worklog.
2. If still under the bars: E0203 near-name fix-its, then the dropped
   block colon cascade; then delete both `#[ignore]`s.
