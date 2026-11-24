# Friction report: noi16p2

## Dream

```cp
let (domain Span, span) = free_spans(bands)          // every band's free spans, top to bottom
let joined = Dsu<Span>()
for (up, down) in bands.adjacent:
    for (a, b) in overlaps(up, down): joined.unite(a, b)
joined.components > 1
```

## Changes

The v0.2 port used `scout.connectivity`, which is gone, read untyped `Int`s, kept a hand
adjacency list of sweep segments with an `add_seg` id counter and a `prev_r = -1` sentinel, and
indexed near cells through a `Set` plus a `Map<(Int, Int), Int>`. Now the reads carry the
statement bounds, rows come from `runs(by: p => p.x)` over the sorted crickets, a row's free
spans are the gaps of `[0] + ys + [m + 1]` taken with `adjacent`, a std `Dsu` joins overlapping
spans of neighbouring bands, the near cells and the cricket set are `compress`ed (sorted and
deduped, looked up with `try_index_of`), and the cut cells come from std `cut_vertices` on
`Graph.undirected`.

## Remaining friction

- std has no "overlapping pairs of two sorted interval lists", so the sweep keeps a two-pointer
  `while` and names spans as `Run(top + i)` from a running offset.
- The near-cell links go through a `List` copied into a typed `[Link]` array for
  `Graph.undirected`.

## Verification

The v0.2 snapshot no longer builds (`scout.connectivity`), so the comparison is with the
accepted C++: 30 random cases agree, corpus tests pass, and --max gives the same output in
0.74 s (accepted C++ 0.87 s, which needs an unlimited stack for its recursive DFS).
