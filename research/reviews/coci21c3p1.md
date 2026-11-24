# coci21c3p1

Score: 2/5
Dream: none

The port directly implements the header's brute-force argument, with bounded reads and a clear periodicity test at lines 14–16. The named optional result and slice output read well, but candidate selection remains a hand-written search and positions have no domain.

## Checklist failures
- idiomatic-code/R3: lines 18–22 hand-roll first-match selection: `for start in 0..n:`, `for len in 1..=(n - start) / k:`, and `if repeats(start, len): return (start:, len:)`. This is not reported as friction.
- idiomatic-code/R6: positions use raw integers: `let a: [n]1..=1000 = read()` (line 12) and `fn repeats(start: Int, len: Int) -> Bool = all(` (line 14), rather than a named lamp-position domain.

## Owed rewrites
- Replace `first_block` (lines 18–22) with one search, `first((start:, len:) for start in 0..n for len in 1..=(n - start) / k if repeats(start, len))`; generator `first` checks on today's `cpl`.
- Introduce a lamp-position domain for the input array and candidate starts; keep lengths as counts.

## Language gaps
None.

## Dream notes
None: there is no `dream.cp` and no `## Dream` in the friction report, so the port is capped at 3.
