# coci10c7p4 frictions

## Dream

```cp
fn delivers(lo: Int, hi: Int) -> Bool:
    let ok = p => lo <= high[p] <= hi
    let seen = reachable(p => p.adj8.filter(ok), from: [post].filter(ok))
    cells.all(p => town[p] != 'K' or p in seen)
```

## Flood fill over grid cells

`reachable` needs a domain for its vertices, and a `(Row, Col)` cell is not one, so the fill keeps
its own stack and `seen` grid (repro sent to Core). There is no spelling for every cell of a grid
either, so `cells` is a comprehension.
