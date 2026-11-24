# apio09p2

Score: 4/5
Dream: 4/5

Cold review of `port.cp` against `dream.cp` and the friction's `## Dream`, without the C++. The
greedy successor links form a forest, std `Lift` climbs them to count how many requests fit a gap,
and free gaps live in a std `IntervalMap<X, ?Int>`. A request is taken iff `a + 1 + b == best`.
That is the editorial's argument.

## Checklist failures
- idiomatic-code/R4: `let starts = [iv[i].l for i in by_start]` (line 20) is a second copy of
  the sorted starts, only to feed `bsearch`. The dream bisects `by_start` directly
  (`by_start.bsearch(i => iv[i].l >= c)`), and the friction does not say why the port differs.
- idiomatic-code/R10: `--max` runs 0.36 s against the C++'s 0.29 s (1.24x, from the friction's
  measurements) with no performance friction. Filed in the friction now.

## Owed rewrites
- Drop `starts` for the dream's `bsearch` over `by_start`, or report why it does not work.

## Language gaps
- `min(by:)` as an op value and `.rev.scan` indexed by the same cuts, for `soonest`: in the
  friction.
- `Lift.of` inferring `V`: in the friction.
- 1.24x at `--max`: added to the friction (perf).

## Dream notes
The friction's dream reads like the editorial. `gap.contains(iv[i])` on a range is invented
(std `Range.contains` takes an element); the port's `gap.clip(iv[i]) != iv[i]` is today's form.
