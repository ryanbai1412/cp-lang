# btoi06p5

Score: pending cold rescore (added dream `proto/ports/scout_batch_48/dream/btoi06p5.cp`, port = dream: derived `escaped_cost`, `plain_cost`, and `SWITCH_COST = 3` in header, named them once, replaced `FAR = 1e18` sentinel with `cost: ?Int` + `enum SetOp: Keep | Forbid | Set(Int)`, fixed `let (n, m)`, replaced `.idx` with `.ord`; verified `cpl run` == `g++`, 32.4 KB, 0 warnings, 0 errors)
Dream: pending cold rescore (new dream matching the polished port)

Baseline review, cold, on main 96359635. The header (lines 1-5) states the DP table and the four operations it needs, and the port does build them as one `LazySegTree` with an assign-then-add `Action` (lines 33-39). It falls short on the cost formulas, which are never derived. No dream exists, so the score is capped at 3.

## Checklist failures

- Run-cost formulas `3 * (k / (n + 2)) + min(3, k % (n + 2))` (line 48) and `3 * ((k + n - 1) / n)` (line 51), and the `+ 3` switch cost (lines 52-57), aren't explained in the header.
- `FAR = 1e18` sentinel (lines 32, 39, 49) where `?Int` in the monoid works.
- Line 7 destructures without `let`: `(n, m): (...) = read()`. Check it against the accepted `let` forms.
- The re-encode loops (lines 70-79) repeat the run-cost formulas from lines 48 and 51 in another form, so each fact lives in two places.
- Mixed `.idx` (lines 25, 67-79) with `.ord` elsewhere in the corpus (§58: one spelling).

## Rewrites owed with today's language

- Name `escaped_cost(k)` and `plain_cost(k)` once and use them in both the DP and the re-encode; derive them in the header.
- Replace `FAR` with `?Int` and `none` as the forbidden mark.

## Gaps that need the language

- None found; the port has no friction report.
