# aac4p5 friction

There is no `dream.cp`; the ideal is below. It is the port except for the two changes named
after it.

## Dream
```cp
let (n, m, x, q): (1..=5e5, 0..=5e5, 1..=5e5, 0..=5e5) = read()
domain V = 1..=n
var dsu = Dsu<V>()
for (u, v) in read<[q](V, V)>(): dsu.unite(u, v)

// merging always grows the room, so keep the x - 1 smallest groups and merge the rest
let groups = dsu.groups().sorted(by: g => g.len)
if groups.len < x: print(-1); exit()
let comps = groups.take(x - 1).collect() + [groups.drop(x - 1).flatten().collect()]

fn edges() -> yields (V, V):
    for c in comps if c.len > 1:
        for (i, v) in c.items: yield (v, c[(i + 1) % c.len])
    for c in comps:
        for (i, u) in c.items:
            for (j, v) in c.items if j != i and j != (i + 1) % c.len: yield (u, v)
    for (i, a) in comps.items:
        for b in comps.drop(i + 1):
            for u in a:
                for v in b: yield (u, v)

let forced = sum(c.len for c in comps if c.len > 1)
// inside a group every ordered pair; across groups half the pairs (earlier to later)
let room = sum(c.len * (c.len - 1) for c in comps) + (n * n - sum(c.len * c.len for c in comps)) / 2
if forced <= m <= room: for e in edges().take(m): print(e) else: print(-1)
```

## The merged components are a function instead of a value
- **Where:** `port.cp`, `fn components() -> List<List<V>>:` that builds `kept` with `var` and
  `push`, called once.
- **What felt wrong:** one value, built once, takes a function, a `var` and a `push`.
- **Goal it hurts:** express.
- **Ideal code:** `let comps = groups.take(x - 1).collect() + [groups.drop(x - 1).flatten().collect()]`
- **General fix:** none needed; list `+` already exists. This is a port rewrite.
- **Severity:** low.
- **Owner:** idioms (this port).

## The room formula is not derived in the header
- **Where:** `port.cp`, `let room = sum(c.len * (c.len - 1) for c in comps) + (n * n - sum(c.len * c.len for c in comps)) / 2`.
- **What felt wrong:** the `Solution:` header says "m fits in the room" but not that the room
  is every ordered pair inside a group plus half of the cross-group pairs (idiomatic-code/R1).
- **Goal it hurts:** evidence.
- **Ideal code:** the comment on `room` in the dream above.
- **General fix:** none; header text.
- **Severity:** low.
- **Owner:** idioms (this port).

## Measurements
- Tokens: port 430, original 1,258.
- `--max` (best of 10): port 0.055 s / 24 MB, accepted C++ 0.108 s / 16 MB. The two print
  different valid edge sets; `checker.py` accepts both.
- Emitted C++: 28,577 bytes.
