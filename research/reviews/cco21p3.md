# cco21p3

Score: 3/5
Dream: 3/5

One DFS gives every tour step the pass it joins, prefix sums find a query's pass, and offline a `Fenwick.kth` finds the step. The header carries the argument, but the corridor order is by index arithmetic and the offline bucketing is by hand.

## Checklist failures

- idiomatic-code/R2: the corridor order is `(start + i) % around[u].len` with `if back == Some(j): continue` (lines 24-26); reported, with the memory measurement behind it.
- idiomatic-code/R6: query ids in `asked` are raw `Int` (`for i in 0..q`, `(k: Int, i: Int)`).

## Owed rewrites

- `domain Query = 0..q` for `asked` and `room`.

## Language gaps

- No linear `position(x)` on a `Seq`, and a bound list copies per frame: research/frictions/cco21p3.md.

## Dream notes

- Two lines (`index_of`, `after`/`before`) cover only the corridor order; the rest of the program has no dream (caps at 3).
