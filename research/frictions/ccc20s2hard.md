# Friction Report: ccc20s2hard

## Graph.of buffers every arc
- **Where:** the hand-built CSR (`at`, `to`, `leads`).
- **What felt wrong:** the graph is one arc per cell, MN = 1.6e7 arcs. `Graph.of(V, arcs)` collects
  the (from, to) pairs into a list before it builds its CSR, about 50% more memory (568 MB measured,
  over the 512 MB limit), so the counting sort is written out.
- **Ideal:** `Graph.of` taking a re-iterable arc source and building in two passes (count, then
  fill), so the port is `reachable(Graph.of(Num, arcs), from: ...)`.

## Dream
```cp
let rows: 1..=4000 = read()
let cols: 1..=4000 = read()
domain R = 1..=rows
domain C = 1..=cols
domain Num = 1..=rows * cols
let room: [R, C]Num = read()
fn arcs() -> yields (Num, Num):
    for (r, c) in (R, C): yield (Num(r.ord * c.ord), room[r, c])
let g = Graph.of(Num, arcs, two_pass: true)
print(yes(g.reachable(from: [room[R.first!, C.first!]]).contains(Num.last!)))
```

- Tokens: port 326, original 815
- Max input (`gen.py --max`): port 0.69 s, 333972 KB, original 1.16 s, 378112 KB
