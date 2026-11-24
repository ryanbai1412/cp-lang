# Friction Report: ioi14p3

The v0.2 port hand-wrote a parent-array DSU with `-1` for a root and two-pass path compression,
kept sizes beside it, threaded all three tables through `mut` parameters, mirrored every count
into `cnt[u, v]` and `cnt[v, u]`, and branched on the first step to ask `READY`. Now std `Dsu`
holds the components (`find`, `size`, `merge` naming the kept root), the asked counts live once
per unordered pair in `[Pair<V>]Int`, a same-component pair says no without touching the table,
and the protocol is one `ask("READY")` and then one `ask(reply)` per later pair. The accepted
C++ is a grader function, so `orig.cpp` adds only a stdio `main`.

## A reply loop needs the first question outside it
- **Where:** `var pair = ask("READY")`, then `pair = ask(answer(..))` for the other pairs, then
  the last answer printed.
- **Ideal:** a callback-style interactive entry, `serve(|u, v| answer(u, v))`, so the program is
  just the rule.

## Dream
```cp
var dsu = Dsu<V>()
var asked: [Pair<V>]Int = 0
serve(fn(u0: V, v0: V) -> Int:
    let (u, v) = (dsu.find(u0), dsu.find(v0))
    if u == v: return 0
    asked[u, v] += 1
    if asked[u, v] < dsu.size(u) * dsu.size(v): return 0
    let (kept, merged) = dsu.merge(u, v)!
    for i in V if i not in [kept, merged]: asked[kept, i] += asked[merged, i]
    1)
```

- Tokens: port 223, original 567
- Max input (`gen.py --max`): port 0.88 s, 12708 KB, original 1.03 s, 12416 KB
