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

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
let n: 2..=3 = read()
domain Row = 1..=n
domain Col = 1..=n
let post = (Row.first!, Col.first!)
let seen = reachable(p => p.adj8, from: [post]).collect()
print(seen.len)
```

```text
error[E0302]: `reachable` (`std.graph`) needs a domain for `V`, and `Int` is not one
  --> probe.cp:5:12
5 | let seen = reachable(p => p.adj8, from: [post]).collect()
  |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  = help: declare a domain, as in `domain V = 1..=n`, and pass its name
```

**Status:** open.
