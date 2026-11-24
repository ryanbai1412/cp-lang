# aac4p4

Score: 3/5
Dream: 4/5

Cold review of `port.cp` against `dream.cp`, without the C++. The dream states the plan once,
`fn plan() -> ?(must: List<Cell>, may: List<Cell>)`, with each case on one line. The port splits
it into two generators so the optional cells stay lazy.

## Checklist failures
- idiomatic-code/R4: `must()` and `may()` (lines 27-44) repeat the case split
  (`if trivial: return`, `if let j = last_f:`, `if l <= n:`). The feasibility test
  `(trivial or last_f != None or s == f)` on line 47 restates it a third time. This is in the
  friction, but the excuse is thin: the dream's 33 MB is well under the 256 MB limit and its
  0.054 s beats the C++'s 0.078 s.

## Owed rewrites
- Take the dream's `plan()`; or try the friction's lazy record
  `(must: yields Cell, may: yields Cell)` and measure it at `--max`.

## Language gaps
None beyond the unmeasured lazy-record plan, which the friction routes to this port.

## Dream notes
One case split, an optional plan and `if let p = plan() and ...`. The `0..=1` banana matrix and
the `take(extra)` fill read plainly.
