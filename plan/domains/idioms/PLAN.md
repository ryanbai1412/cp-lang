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

## General fixes owed
Each local fix tonight, with the general fix that would remove its whole class (Ryan, 2026-11-22).

| Instance (local fix) | General fix | Owner |
|---|---|---|
| cf2147f, ucup5s05f, the chunk_by/scan_in_place/next_less migrations: renamed or removed stdlib names fixed by hand in each port | Every removed name keeps a tombstone that turns E0201 into "renamed to X" with a machine-applicable fix, so `cpl lint --fix` (or a `cpl migrate` codemod) moves old code, ports and docs examples together | stdlib (tombstones), diagnostics (fix), lint-fmt (codemod) |
| noip21p3: `scan(gaps, +)[i]` returned the element, not the fold, so the port was rewritten to `prefix_fold` | One indexing rule for every fold/view type, checked by a conformance case per type, so a view can't silently index its source | spec + compiler |
| `.rev` view indexing (P0, run 1) | Same conformance family: every view's `[i]` agrees with its `print` order | compiler |
| `count(<bool gen>)` counts every element (P0, run 1) | The free fold forms (`sum`/`count`/`any`/`all`/`max` over a generator) share one implementation with the method forms, and a test checks each pair | stdlib |
| `Map.get` copies its value, a hidden O(n) (P0, run 1) | Getters return a borrowed view by default and a copy only when asked; a perf lint flags a value-type copy in a loop | compiler (codegen), lint-fmt |
| web/content/docs/guides/stdlib.mdx lists removed APIs | Generate the stdlib tables from `std/*.cp` exports and doc comments, checked in `gen.toml` | site + stdlib |
| ci-steps.sh's hand-rolled LSP check broke twice (no `timeout` on macOS, a split read on Windows) | Done: the acceptance job runs the repo's cross-platform `editors/lsp-smoke.py` (diagnostics, completion, definition, format) | idioms |
| Each held-out problem has its own brute/gen/check scripts | One problem-package format shared by corpus/, research/acceptance/ and `cpl stress`/`cpl test` (gen, brute, checker, limits), so a judge is written once and every tool runs it | idioms + debugger + tools |
| newcomer `score.py` has to match a lint to a mark within ±3 lines | Every lint anchors on the construct it names (the loop header, the fn), documented in the catalog schema, so marks match exactly | lint-fmt |
| `cpl test` case-sensitive `Yes`/`YES`; any-answer problems need a hand-written `@check` | The output comparison mode (tokens, case-insensitive yes/no, floats with tolerance, checker) is part of the problem package and judge profile, not per-solution code | tools |
| Run 1's 2 h virtual round took the agent 2 minutes | Score agent runs by findings and tool round-trips, not wall-clock, and pace the round by problem count | idioms |
| CI gate red on `skill-check: stale generated file` (bef33ca03) until the regen job caught up; direct `--no-verify` pushes skip the pre-commit gen hook | The gate treats a stale generated file as "regen pending" (or runs the writers itself) when the regen job will land it; staleness stays a failure only on the regen job | infra |
| port-check burn-down: 305 ports fail; `cpl migrate` + `cpl fix` recover only 11 | E0208 (uppercase let) and `V.first` (?V) get machine fixes, and every removed std name gets a migrate rule, so one `cpl migrate && cpl fix` pass moves stale ports, and old friend code, forward | diagnostics (fixes), stdlib (migrate rules) |
| Ryan's standard port header (Source, Problem, Solution) | `corpus/header.py` counts coverage (0/877 today); headers are added longest first, and new ports start from the template in plan/PORTING.md | idioms |
