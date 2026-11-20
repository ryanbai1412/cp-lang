// SAC '22 CC5 P5 — original: corpus/submissions/4575631.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

let (n, m, k): (1..=5_000, 1..=5_000, 1..=5_000) = read()
domain Row = 0..n
domain Col = 0..m
let g: [Row, Col]Char = read()

// FRICTION 39.8: RollingHash is documented for Str only; 2D hashing needs
// RollingHash over [Row]U64 (or HashVal) and a named return type for .get(l..r).
let row_h = [Row] r => RollingHash(g.row(r))
var seen: List<U64> = []

for c in 0..=m - k:
    let col_vals = [Row] r => row_h[r].get(c..c + k)
    let col_h = RollingHash(col_vals)
    for r in 0..=n - k:
        seen.push(col_h.get(r..r + k))

sort(mut seen)
dedup(mut seen)
print(seen.len)
