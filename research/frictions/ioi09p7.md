# Friction report: ioi09p7

There is no dream.cp; the ideal is below. The port is close to it. The sqrt split is the editorial's: the big regions' sweep is cached as `?[Region]Int`, and a small region's `SortedSet.range(t.sub(a)).len` is memoized with `??=`.

## Dream

```cp
let (n, r, q): (1..=2e5, 1..=25000, 1..=2e5) = read()
domain V = 1..=n
domain Region = 1..=r
domain Staff = 2..=n
let head: Region = read()
let staff: [Staff](boss: V, home: Region) = read()
let t = Rooted.from_parents([V] v => Staff.at(v).map(s => staff[s].boss))
let home = [V] v => Staff.at(v).map(s => staff[s].home) ?? head
let spots = [Region] x => SortedSet([t.pos(v) for v in V if home[v] == x])
...
```

## The supervisors are read row by row into two `var` arrays
- **Where:** lines 337-344: `var home: [V]Region = [V] _ => head`, `var boss: [V]?V = None`, then `for v in V.all.drop(1): let (b, h): (V, Region) = read()` and two stores.
- **What felt wrong:** one input table is spread over a loop, two mutable arrays and a default. Every employee but the first has a row, which is a domain, not a loop.
- **Goal it hurts:** express.
- **Ideal code:** the dream's `staff: [Staff](boss: V, home: Region) = read()` over `domain Staff = 2..=n`.
- **General fix:** none needed in the language: a typed read over a sub-domain already compiles (btoi17p6 reads `[Kid]V` over `domain Kid = 1..n`). It is a rewrite owed by the port, and a lint for "a loop of `read()`s stored into parallel arrays" would catch the class. Owner: lint.
- **Severity:** low.

## Measurements

- **Tokens:** port 423, accepted C++ 785 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.98 s / 27 MB, accepted C++ 1.09 s / 32 MB, with the same output.
- **Emitted C++:** 30789 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
