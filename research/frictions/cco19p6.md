# Friction report: cco19p6

## Dream

```cp
domain Tail = (Word, Cut)
let dist = dijkstra(t => Word.filter_map(x => lay(t, x)), from: starts)
print(Tail.filter_map(t => if ended(t): dist[t] else: None).min() ?? -1)
```

## Hand Dijkstra over strings becomes std `dijkstra` over typed tails

The v0.2 port keyed a `Map<Str, Int>` of distances by the tail text and ran a hand heap loop
with an `Int.MAX` sentinel and a `-1` answer variable. A tail is always a suffix of a
codeword, so the rewrite names it as a product domain `Tail = (Word, Cut)`, lists its arcs
once with `lay`, and runs std `dijkstra` over `Graph.of(arcs)` from every overlapping pair.
The answer is the nearest empty tail, `-1` only at print.

## Remaining friction

`dijkstra` takes a graph, not a `Tail -> arcs` function, so the arcs are materialised in a
list first (n * (m + 1) * n of them, fine here).

## Verification

`gen.py` is new (words built from a few short pieces so prefixes are common). 30 random cases
agree with the v0.2 port and 30 with the accepted C++. Corpus tests pass.
