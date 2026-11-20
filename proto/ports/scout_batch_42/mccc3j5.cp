// Mock CCC '22 Contest 1 J5 — Maximum Independent Set via Subset<V> (CP Lang v2 spec)
// original: corpus/submissions/4223449.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Given n <= 20 vertices and m conflict edges, find the largest subset of vertices
// containing no edge. Rethought with `Subset<V>` instead of raw bit shifts and `goto`.

let (n, m): (1..=20, 0..=200) = read()
domain V = 1..=n
let edges: [0..m](u: V, v: V) = read()

// FRICTION MC7: `Subset<V>` documents `for g in s.subsets`, `.with(v)`, `.len`,
// and `v in s`, but specifies no constructor for the FULL set (`Subset<V>.full`
// or iterating `for s in Subset<V>` directly as an `Index` type), forcing either
// a manual fill loop to build `full` or falling back to raw `0..(1 << n)` masks.
var full = Subset<V>()
for v in V: full.add(v)

let ans = max(
    s.len
    for s in full.subsets
    if all(not (u in s and v in s) for (u, v) in edges)
) ?? 0

print(ans)
