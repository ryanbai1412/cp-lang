# atcoder_dp_n

Score: 3/5
Dream: 4/5

Cold review of `port.cp` against `dream.cp`, without the C++. `cost: [I.ranges]Int` with
`min(cost[x] + cost[y] for (x, y) in r.splits)! + size.fold(r)` is the editorial's recurrence
word for word; `prefix(a)` gives range sums.

## Checklist failures
- idiomatic-code/R10: 1.8x the accepted C++, from the same range-table layout and index code as
  atcoder_dp_l (routed to Core). Nothing in the port to change.

## Owed rewrites
None in the port.

## Language gaps
- `[V.ranges]` layout and split indexing (Core).

## Dream notes
The dream uses `.map(...).min()` where the port's comprehension `min(...)` is the corpus
spelling (idiomatic-code/R8); 4/5 until the dream takes the port's line.
