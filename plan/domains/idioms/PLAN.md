# idioms plan

Detail behind [README.md](README.md). Numbers are from [metrics.toml](metrics.toml) (2026-11-22).

## Where we stand
- v0.3 can express the 20 dry-run problems (20/20 `loop.sh`), but 15 needed a `_w` workaround variant along the way, and the solutions were written with the compiler in the loop.
- The skill exists only on `devin/skill-v2` (check passes there). Main ships no skill.
- The 305 tracked ports are pre-0.3 code: 175 check on v0.3, 0 are judge-proven, and their review scores were given under 0.2.
- The corpus suite moved 7 -> 12/100 since the committed scoreboard; 46 of 100 have no tests yet.
- The token-ratio script broke when `scripts/scoreboard.py` was deleted (`ea2b88ff`), so goal 1's token test has no measurement today.

## Workstreams, in order (Milestone 1 first)
0. **Milestone 1 acceptance run** (first; due about 09:30 PST 2026-11-23). Done: the protocol, the run schema, the `BLOCKERS.md` generator and its `gen.toml` entry, and the held-out problems. Still to do: local judges (reference, gen, checker) for the 15 problems; a cold agent on Linux playing the friend with only the site, Neovim and Codeforces statements; CI runners for install and CLI on macOS and Windows; Codeforces-profile builds checked at 64 KB. A run every 2 hours, then BLOCKERS.md regenerated, each lane sent its rows, and a worklog checkpoint. No real judge submissions. This replaces workstream 3's unseen-20 gate for now.
1. **Land in-flight work** (done 2026-11-22: skill, eval, principles and catalog on main).
2. **Measurement repair** (done: token ratio, judge coverage of `corpus/sets/v03.txt`). Kept only as the Submit-step evidence.
3. **Blind eval loop.** Rerun the known-20 cold eval when the skill changes, as a regression check. It feeds the acceptance run's agent setup, which uses the docs, not the skill.
4. **Re-ports.** Mechanical migration continues only when a stdlib or compiler fix unblocks a port (16 left, each filed with its owner). The idiomatic rethink from `dream.cp` is deferred until the milestone passes.
5. **Corpus growth.** Deferred, except for problems that become acceptance-run material.

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

## Next moves (Milestone 1)
1. Local judges for the 15 held-out problems (`research/acceptance/problems/<id>/`).
2. Score run 1, publish BLOCKERS.md, send each lane its rows; a run and a worklog checkpoint every 2 hours.
3. A Linux, macOS and Windows CI job for the install and CLI steps, feeding each run.
