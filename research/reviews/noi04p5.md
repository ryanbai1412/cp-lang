# noi04p5

Score: 2/5
Dream: 3/5

Port 2/5, dream 3/5 (the friction's `## Dream`, one declaration). The DP over (south patches placed, width used, north patches used) with a precomputed even split of a south run (lines 21-40) is the editorial's, in tenths so the cost stays integral.

## Checklist failures

- idiomatic-code/R4, R1: the hut's width 100 is written six times (lines 26, 29, 34, 37, 41) and named nowhere; not in the friction.
- idiomatic-code/R6: `[0..=n, 0..=100, 0..=m]` (line 29) and `[1..=100, 1..=100]` (line 26) are anonymous axes, so `cost[i, k, j]` and `south[len, parts]` rely on argument order.
- The slopes are read as `F64` and rounded to tenths (lines 15-16), and the answer is printed by hand from tenths (line 42): reported as the missing `Decimal<1>`.

## Owed rewrites

- `let width = 100` (or `domain Width = 0..=100`) and named axes for patches and width.

## Language gaps

- No fixed-point decimal read/print: in the friction.

## Dream notes

`Decimal<1>` fixes the read only; the dream should also show the DP over named axes and the decimal print.
