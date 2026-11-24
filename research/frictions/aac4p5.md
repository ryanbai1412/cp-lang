# aac4p5 — Neighbourly Neighbourhoods

Keep the x - 1 smallest groups of the forced pairs, merge the rest, then print the first m
edges of a lazy stream: the cycles, the other pairs inside groups, pairs from earlier to later
groups.

## Dream

```cp
let (n, m, x, q): (1..=5e5, 0..=5e5, 1..=5e5, 0..=5e5) = read()
domain V = 1..=n
var dsu = Dsu<V>()
for (u, v) in read<[q](V, V)>():
    dsu.unite(u, v)

// Merging always grows the room, so keep the x - 1 smallest groups and merge the rest.
let groups = dsu.groups().sorted(by: g => g.len)
fn components() -> List<List<V>>:
    var kept = groups.take(x - 1).collect()
    kept.push(groups.drop(x - 1).flatten().collect())
    kept

// Cycles first (each group of two or more must be strongly connected), then the other
// ordered pairs inside a group, then pairs from an earlier group to a later one.
fn edges(comps: List<List<V>>) -> yields (V, V):
    for c in comps if c.len > 1:
        for (i, v) in c.items:
            yield (v, c[(i + 1) % c.len])
    for c in comps:
        for (i, u) in c.items:
            for (j, v) in c.items if j != i and j != (i + 1) % c.len:
                yield (u, v)
    for (i, a) in comps.items:
        for b in comps.drop(i + 1):
            for u in a:
                for v in b:
                    yield (u, v)

if groups.len < x:
    print(-1)
else:
    let comps = components()
    let forced = sum(c.len for c in comps if c.len > 1)
    let room = sum(c.len * (c.len - 1) for c in comps) + (n * n - sum(c.len * c.len for c in comps)) / 2
    if forced <= m <= room:
        for (u, v) in edges(comps).take(m):
            print(u, v)
    else:
        print(-1)
```

## `take` on a generator is not lazy
- **Where:** `port.cp`, `fn edges` builds a `List` and checks `if out.len == m: return out`
  before every push, where the dream yields every edge and calls `edges(comps).take(m)`.
- **What felt wrong:** the edge stream has up to n^2 = 2.5e11 items. The stopping rule belongs
  to the caller (`take(m)`), but it has to be threaded through every loop of the producer.
  Repro (sent to Core):
  ```cp
  fn nat() -> yields Int:
      var i = 0
      while true:
          yield i
          i += 1
  print(nat().take(3).collect())
  ```
  never returns under `cpl run` or the built C++. On the dream's max case the C++ dies with
  `std::bad_alloc`.
- **Goal it hurts:** the obvious way is the fast way; express.
- **Ideal code:** the dream above.
- **General fix (core, generators):** `take`, `first`, `find` and `any` on a `yields`
  generator stop pulling once they have their answer.
- **Severity:** high (the obvious spelling runs out of memory).
