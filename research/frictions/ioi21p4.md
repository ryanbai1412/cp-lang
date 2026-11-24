# Friction report: ioi21p4

## Dream

```cp
enum Base: A | C | T
@grader fn init(a: [Pos]Base, b: [Pos]Base)
let count = Prefix.count_by((a, b))              // [Base, Base]Prefix<Pos, Int>
@grader fn get_distance(x: Pos, y: Pos) -> ?Int:
    let have = count.fold(x..=y)
    if any(have.row(s).sum != have.col(s).sum for s in Base): return None
    let swaps = sum(min(have[p.lo, p.hi], have[p.hi, p.lo]) for p in Pair<Base>)
    swaps + (have.off_diagonal.sum - 2 * swaps) / 3 * 2
```

## Letters as a domain

`Base` is `0..3` with a hand `base(c)` map, because a `Char` enum (`A | C | T`) can't be read
from a string and used as a table index. Reading `[Pos]Base` straight from the string would
drop `base` and the `Str` indexing by `Int(i)`.

## Prefix counts by key

`count` builds nine `Prefix`es, each from its own indicator table. A `count_by` that keys
prefix counts by a value (or a `Prefix` over `[Base, Base]Int` cells) would be one call, and
the fold would give the whole `have` table at once.

## Row and column sums

`sum(have[s, t] for t in Base)` against `sum(have[t, s] for t in Base)` spell a row and a
column sum; `have.row(s).sum` / `have.col(s).sum` would name them.

## The grader lives in the solution

`init` / `get_distance` are grader functions; the port reads their arguments from stdin, and
`-1` stands for "impossible" because the grader's protocol says so.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (`orig.cpp`, the AC
grader functions plus a stdin `main`); the max case (n = q = 1e5, half the queries on
rotated aligned triples) is byte-identical to the C++ and runs in 0.03 s.
