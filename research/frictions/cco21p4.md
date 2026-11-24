# cco21p4 friction

The old port read `(n, m): (Int, Int)` and unbounded `r`, `p`; built the reverse graph by
hand from a comprehension; kept a separate `dead` mask and a queue head; wrote the
"keep the smaller of an optional" update twice as nested `if let`; and sorted
`(-r, index)` pairs to get roads by descending `r`. The port types every bound, uses
`Graph.directed(roads).transpose()` and `g.deg`, writes the single `drop(road, price)` step
both phases share, updates with `ans[u] min= price` (None is the identity), sorts the road
domain with `desc`, and prints with `none: -1`. The problem had no generator; `gen.py` is
new.

## Dream

The existing `dream.cp` uses a `Settle<roads.keys, Int>(best: max)` queue that does not
exist in std and reads as a different algorithm; the port above is the better target:

```cp
for e in Road.sort(by: e => desc(roads[e].r)):
    drop(e, Some(roads[e].r))
    peel()
```

No gaps between the port and that shape.

Measured: `cpl stress` agrees with the old port and with the accepted C++ (6419583) on 30
cases each; on the `--max` case (n = m = 2e5) the port gives the C++'s output in 0.07 s
against its 0.12 s.
