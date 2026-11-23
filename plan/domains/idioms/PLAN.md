# idioms plan

Detail behind [README.md](README.md). Numbers are from [metrics.toml](metrics.toml) (2026-11-22).

## Where we stand
- v0.3 can express the 20 dry-run problems (20/20 `loop.sh`), but 15 needed a `_w` workaround variant along the way, and the solutions were written with the compiler in the loop.
- The skill exists only on `devin/skill-v2` (check passes there). Main ships no skill.
- The 305 tracked ports are pre-0.3 code: 175 check on v0.3, 0 are judge-proven, and their review scores were given under 0.2.
- The corpus suite moved 7 -> 12/100 since the committed scoreboard; 46 of 100 have no tests yet.
- The token-ratio script broke when `scripts/scoreboard.py` was deleted (`ea2b88ff`), so goal 1's token test has no measurement today.

## Workstreams, in order
1. **Land in-flight work** (phase 2 day 1). Rebase `devin/skill-v2`, `devin/skill-eval-1`, `devin/cp-lang-skill` (drop if `git cherry` shows it superseded by skill-v2), the docs part of `devin/idioms-docs`. Add `skill-gen.py --check` to the fast gate (ask infra). Findings from `research/skill-eval/repro/` go to compiler/stdlib/site worklogs as requests.
2. **Measurement repair.** Restore token counting (port it into `cpl-conform` or a small `scripts/` helper with infra's agreement); `cpl-conform corpus --write` so the board matches a fresh run; make `judges.rs` read the scoreboard as JSON instead of slicing strings; add the tracked ports that pass check to the judge proof.
3. **Blind eval loop.** One cold run per skill change on the 20; then write a fresh unseen 20 (CSES/AtCoder ABC-E/F style, same shape as `research/dryrun/`: ref, gen, tests). Each run records pass, workaround, lint per problem in `research/skill-eval/`.
4. **Re-ports.** Batches of ~20: migrate `port.cp` to v0.3 (judge proof), then rethink from `dream.cp` in review-score order (18 under 4 first). Each port lands with its friction report and review. Lint warnings on re-ports are triaged; false positives go to lint-fmt as FP cases.
5. **Corpus growth.** Hill-climb beyond 100 in thin areas (interactive, geometry, strings, flows); every dream/spec gap in `plan/domains/spec/delta/hillclimb.md` becomes a spec proposal or a conformance case.

## Why these milestones
- **M1 (skill on main, blind-measured)** first because Ryan called the skill critical, and every other eval (writable-from-docs, re-ports by agents) runs through it. Landing is mostly done work.
- **M2 (corpus measured on every judge)** second because the domain's claims rest on numbers that are stale (scoreboard) or unmeasurable (tokens); and judge proof needs no Ryan time.
- **M3 (idiomatic re-ports)** last: it is the largest and needs Ryan's taste calibration (sample reviews), which the M1 principles make cheap.

## Alternatives considered
- *Re-port everything from scratch now.* Rejected: no token or judge measurement yet, and the skill that would guide agents is not landed; we would re-port blind.
- *Keep the skill outside the repo (site-only).* Rejected: `skill-gen.py --check` ties every cp block to the checker; outside the repo it drifts.
- *Judge-prove by running real judges.* Not possible offline; `judges.rs` compiles with each judge's flags and stack, which is the closest local proof. Real submissions stay a sample (Ryan or contest lane).
- *Delete the 130 tracked ports that fail check.* Rejected: they are the evidence of what v0.3 cannot yet say; each becomes a re-port.

## Risks
- **Eval leakage.** The dry-run 20 are known to the skill writers, so 20/20 can overfit. Mitigation: a fresh set per release (QUESTIONS).
- **Skill drift.** Stdlib APIs change under the skill. Mitigation: generated `stdlib/*.md` and `--check` in the gate.
- **Ownership seams.** Review, corpus and token scripts live in infra's `scripts/`; `cpl-conform` has no named owner. Requests filed, not worked around.
- **Taste mis-calibration.** Agent reviews score 4 by default; without Ryan's samples the rubric drifts. Mitigation: Ryan reviews 5 re-ports per batch.
- **Gate cost.** Adding 305 ports to `judges.rs` slows the fast gate; keep the full sweep in `--full` and a sampled set in `--fast`.

## Phase 2, first moves
1. Rebase and land `devin/skill-v2` (check + fast gate), then `devin/skill-eval-1`, then the `devin/idioms-docs` docs.
2. Ask infra to gate `skill-gen.py --check` and to own a token-count helper; file the 5 repro findings with their owners.
3. Refresh the corpus scoreboard and extend `judges.rs` to tracked ports that check.
4. Run one blind eval on the landed skill and record it.
