# egoi22p7

Score: 3/5
Dream: none

Cold review of `port.cp` (no `dream.cp`, no friction `## Dream`, so capped at 3). Designs and representatives keyed by span, joined by halves, with the grader as a `List<Dsu<Pin>>`: the header's divide and conquer without heap numbering.

## Checklist failures
- idiomatic-code/R7: line 56, `if connected(d, x, rep[left]) == d: ...`, counts a query and may push a design inside a condition; line 73 binds `let wired = connected(...)` first, as line 56 should.
- idiomatic-code/R6: `type Span = (lo: Int, hi: Int)` (line 37) keeps pins as `Int`, so `Pin(s.lo)` and `Pin(i)` convert back at lines 46, 49, 61 and 70.

## Owed rewrites
- Bind the query at line 56 before branching.
- Make `Span` a `Range<Pin>` (or `(lo: Pin, hi: Pin)`) with the midpoint in `halves`.
- Write the dream.

## Language gaps
None shown.

## Dream notes
No dream.
