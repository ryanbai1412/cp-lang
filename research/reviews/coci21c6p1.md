# coci21c6p1

Score: 3/5
Dream: none

The port directly implements the header's extremal-ranking argument: give the chosen keeper all 500 points for the best place, and give everyone else those points for the worst. The ordering is expressed once through tuple comparison in `place` (lines 21–24), and `for k in Keeper: print(place(k, 500, 0), place(k, 0, 500))` (line 26) reads like the editorial (idiomatic-code/R2). The missing dream caps an otherwise compact port at 3.

## Checklist failures
- Porting workflow: no `dream.cp` and no `## Dream` in the friction report. The dream is missing; the required independent editorial target cannot be reviewed.

## Owed rewrites
- Read the keepers as one table, `let keepers: [Keeper](name: Str, rounds: [5]0..=500) = read()`, and derive `total` as `[Keeper] k => keepers[k].rounds.sum()`, replacing the `var` fill loop at L13–18 (the struct-row read checks and runs on today's `cpl`).
- Supply the missing dream.

## Language gaps
None. The friction reports no language gap, and the port shows none requiring a workaround.

## Dream notes
None: there is no `dream.cp` and no `## Dream` in the friction report, so the port is capped at 3.
