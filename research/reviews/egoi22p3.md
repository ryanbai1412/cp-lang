# egoi22p3

Score: 3/5
Dream: 3/5

Cold review of `port.cp` against `dream.cp`. `Rooted.forest` over `g.only(...)` and `forest.path(a, b)` (lines 20-40) are the editorial's DFS forest and paired paths; `pair_up` returns the friend left open, as the header says.

## Checklist failures
- idiomatic-code/R7: line 43, `if any(pair_up(r) != None for r in forest.roots if r != home):`, reads as a parity test but writes every `route` (lines 39-40).
- idiomatic-code/R2: lines 37-38, `for i in 0..open.len / 2:` and `(open[2 * i], open[2 * i + 1])`, index pairs by hand; in the friction.

## Owed rewrites
- Run `pair_up` over the roots in a plain loop, then test the results, so route building is a statement.

## Language gaps
- No exact-size chunks: `open.chunks_exact(2)` is `error[E0203]: a List<Int> has no method chunks_exact` on a fresh `cpl`; repro now in the friction.

## Dream notes
The dream drops the index arithmetic with `chunks(2)` and an `a != b` guard, but hides the same route writes in `filter_map(pair_up)` and `forest.roots.all(...)`, so it keeps the R7 failure.
