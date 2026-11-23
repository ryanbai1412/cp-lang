# diagnostics

Segment: B. Metric. Owns: `crates/cpl-diag`, `spec/diagnostics.md`, the
error sites in the front end (with compiler). Detail: [PLAN.md](PLAN.md) ·
[ARCHITECTURE.md](ARCHITECTURE.md) · [metrics.toml](metrics.toml) ·
[worklog.md](worklog.md). Numbers measured on main at 4b8f504f; every one
has its command in `metrics.toml`.

## Mission
One error per root cause, naming the place, the rule and the fix, so a
broken program costs the contestant one read and (where the fix is
obvious) one keystroke. Runtime traps name source values and locations.

## Covers
Error codes and their catalogue (`spec/diagnostics.md`, `cpl explain`),
message wording, spans, parser and checker recovery, cascade suppression,
fix-its (`cpl fix`, LSP quick fixes), runtime trap messages, the
broken-ports recovery scoreboard.

## Not covered (owner)
- What is an error (the rules themselves): compiler and spec owners.
- Lint rules and their wording: lint-fmt (codes still go in the catalogue).
- Editor presentation of diagnostics: tools (LSP) / editors.
- Trace and debugger views of a failure: tools.
- Error-index pages on the web site: site (none today; nothing under `web/` reads `REGISTRY`).

## Success criteria for v0.3
1. Broken-ports benchmark: first error on the broken line >= 99.5%,
   cascades <= 2%, silent 0, both scoreboards gated in `check.sh`.
2. Common mistakes (`&&`, `!x`, `let` for `var`, typos ...): >= 95% carry a
   fix-it and >= 95% of fixes restore a clean program.
3. Catalogue is the truth: every emitted code is a `codes::` constant with a
   registry trigger, an example message and a conformance case; no code is
   marked *(0.3)* while emitted.
4. No regression on the guards: front end < 50 ms on 5000 lines, budgets.

## Milestones
**M1. Gated recovery (phase 2, first).** Exit: `cargo test -p cpl-cli --test
recovery -- --ignored` passes with the current bars (>= 99.5% first-right,
<= 5% cascaded, >= 88% fixable), the two `#[ignore]` lines are deleted, and
`scripts/check.sh` runs it; raw `Code("E…")` literals = 0.

**M2. Fixes everywhere obvious.** Exit: common-mistake fix-it >= 95% and
restore >= 95%; broken-ports fix-it >= 80%; catalogue rows without an
example = 0; generic `expected ...` first errors on spec blocks <= 20;
bars in `recovery.rs` raised to these numbers.

**M3. Root-cause clean.** Exit: broken-ports cascades <= 2%;
common-mistake cascades <= 3; benchmark base = every clean `spec.cp`
(not a hand list); golden errors with help >= 90%.

## Metrics (main at 80a47142 → target; packet was at 4b8f504f)
| Metric | Today | Target | M |
|---|---|---|---|
| Broken ports: first error on broken line | 100.0% (414) | >= 99.5% | M1 |
| Broken ports: cascaded | 0.2% (1/414) | <= 2% | M1/M3 |
| Broken ports: silent | 0 | 0 | M1 |
| Broken ports: fix-it | 76.6% | >= 80% | M2 |
| Common mistakes: fix-it | 100.0% (106/106) | >= 95% | M1/M2 |
| Common mistakes: fix restores | 97.2% | >= 95% | M1/M2 |
| Common mistakes: cascaded | 1/107 | <= 3 | M3 |
| Spec blocks: first error generic `expected` | 11/68 (API synopses skipped) | <= 20 | M2 |
| Scoreboards gated | 2 of 2 | 2 of 2 | M1 |
| Benchmark base (clean `spec.cp`) | 37 of 100 | all clean, auto | M3 |
| Emitted codes with trigger + clean fix | 83 of 84 | all | M1 |
| Emitted codes whose trigger has a fix-it | 25 of 84 | >= 45 | M2 |
| Catalogue rows without example | 0 of 93 | 0 | M2 |
| Raw `Code("…")` sites outside `codes::` | 2 (lint, LSP) | 0 | M1 |
| Codes with no conformance case | 2 | 0 | M2 |
| Golden errors with help | 38/58 | >= 90% | M3 |
| Front end cold, 5002 lines (release) | 34 ms | < 50 ms | guard |
| Budgets (`run` < 200 ms, `build` < 1 s) | 2/2 | 2/2 | guard |

The scoreboard commands are `cargo test -p cpl-cli --test recovery --
--ignored --nocapture`; it fails on main today, which is why it is ignored.

## Judgment rubric (what numbers miss)
1. Does the first error name the rule in the contestant's words, not the
   parser's (`expected an expression` vs "this `=` needs a value")?
2. Does the fix-it do what a strong contestant would do, never a
   placeholder that compiles but is wrong?
3. Is the primary span the place to edit, and does a label point at the
   other half (declaration vs write)?
4. Does every message use the spec's vocabulary (one spelling per idea)?
5. Would the contestant trust a trap message enough to fix without the
   debugger?

## Probes (a cold agent with only the docs and `cpl`)
- Break 10 unseen ports by hand in ways a human would at 3am; fix each from
  the first message alone; score fixes needing more than one read.
- Given 20 random errors from the scoreboard, rewrite any message whose rule
  is unclear; the count of rewrites is the rubric score for Q1/Q4.
- Apply every offered fix-it on the planted bugs; any wrong-but-compiling
  result fails Q2.
- Plant 10 runtime bugs (overflow, out of range, empty pop); fix from the
  trap message alone (Q5).

## Improve (highest impact first)
1. Land `devin/wip-diag-recovery` (typo fix-its, `!in`, unclosed parens,
   error-typed `??` suppression), re-measure, un-ignore the scoreboards.
2. E0203 "no std module declares it": suggest the near name (58 of 142
   common-mistake first errors, 23 with a fix).
3. Dropped block colon: 33 of 43 broken-port cascades (mostly E0006).
4. `let` for `var`: 11 cascades; fix at the declaration, label the write.
5. Raw code literals and *(0.3)* lies in the catalogue; fill 33 examples.
6. Generic `expected ...` messages on spec blocks.

## Ryan's role
Signs off once: the targets above and a one-page message style guide
(phase 2 drafts it from the rubric). After that, metric-driven.

## Decisions for Ryan
1. Fix-its only where the fix is genuinely right, never placeholders
   (`let x = todo`) to raise the rate? In [QUESTIONS.md](../../QUESTIONS.md).
2. Are the targets right: cascades <= 2% (stricter than the current 5% bar)
   and common-mistake fix-it >= 95%?
3. Scoring `let` for `var`: count the error at the write as right when it
   labels the declaration (today it scores 0% first-right)?
