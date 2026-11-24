# Friction report: ports landing without a friction report

## A port can land with `Skipped: friction report and review`, and nothing picks it up
- **Where:** 37a647a98 (ioi25_festival), 67a6c650c (ioi25_migrations) and 51b202308
  (ioi25_worldmap), each with `Skipped: friction report and review, the task limits changes to
  the package.` The festival and world map reports were written later by hand; migrations had
  none until its review asked for one.
- **Repro:** `git log origin/main --grep='Skipped: friction' --format='%h %s'` lists the three.
  Today 258 of the 873 `corpus/problems/*/port.cp` have no `research/frictions/<id>.md`, 100 of
  them ports newer than `research/reviews/baseline.json`. `scripts/review-check.py --check` fails
  a new port with no `dream.cp` or no review, but not one with no friction report.
- **What felt wrong:** the skip reason is a brief's scope ("changes to the package"), not
  process costing more than 10% of the work (handbook/process/handbook-and-rfcs.md, the escape
  hatch). The friction report is how a port's gaps reach their owners (porting step 4), so a
  skipped one drops them silently until a reviewer or a triage pass digs them out of the diff.
- **Goal it hurts:** evidence, accumulate.
- **Ideal code:** every new port lands with its friction report, or with a skip that a gate
  turns into a tracked item.
- **General fix (factory):** make `review-check.py --check` also fail a new port with no
  `research/frictions/<id>.md` (a ratchet over today's 100, as `new_under_4_max` does for
  scores), and have package-only briefs name `research/frictions/` and `research/reviews/` as
  in scope. This covers every port lane.
- **Owner:** Factory.
- **Severity:** medium.
