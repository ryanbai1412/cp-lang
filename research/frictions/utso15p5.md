# Friction report: utso15p5

There is no dream.cp; the ideal is below. The port:
- runs Kruskal with a `Dsu` (`unite` returning whether it joined);
- roots the MST;
- keeps each tree road at its lower town in a `SegTree` of the two largest distinct costs;
- folds every road's path over `tree.chains(u, v, edges: true)`.

## Dream

```cp
type Top = (hi: ?Int, lo: ?Int)                  // None below every cost
let tree = Rooted(Graph.undirected(kept), Town.first!)
...
if let out = (if path.hi < Some(cost): path.hi else: path.lo): best min= base + cost - out
```

## 0 is the "no cost" identity of the two-largest monoid
- **Where:** line 25: "0, below every cost, is max's identity". `(hi: 0, lo: 0)` is used four times, and `if out != 0` on line 53 tests for it.
- **What felt wrong:** costs are `1..=1e4`, so 0 works, but it is a sentinel (R5). A zero-cost road in a variant would silently break `out != 0`.
- **Goal it hurts:** hard-to-get-wrong.
- **Ideal code:** the dream's `?Int` fields, with `None` ordered below every value.
- **General fix:** none in the language: `?Int` ordering and `max` over optionals exist. The rewrite is owed by the port; it costs a wider `Top` in the SegTree. Owner: idioms.
- **Severity:** low.

## A list is copied into an array to give it a domain
- **Where:** lines 30-31: `domain Link = 0..kept.len` and `let links = [Link] i => kept[i.idx]`, only so that `Graph.undirected` gets an edge array.
- **What felt wrong:** `kept` is already the edge list.
- **Goal it hurts:** express.
- **Ideal code:** `Graph.undirected(kept)`.
- **General fix:** let `Graph.undirected`/`directed` take a `List` of edges, indexing it by a fresh domain. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 430, accepted C++ 1383 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.05 s / 12 MB, accepted C++ 0.04 s / 18 MB, with the same output.
- **Emitted C++:** 30470 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
- **Speed gap:** 0.05 s vs 0.04 s at `gen.py --max` is over 1.1x. The repro (this package's `--max` case) is sent to perf via Factory 7bd146f2. Owner: Perf B (33c55557).
