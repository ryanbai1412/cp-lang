// Codeforces 2201 F1 — Conjugate Row-Count Multiset Hashing (Easy Version)
//
// Maintain two Ferrers diagrams (`Left` and `Right`), where `row[s, r]` counts
// cells added to row `r` on side `s`, and `freq[s, f]` counts rows on side `s`
// with at least `f + 1` cells (the `(f + 1)`-th column length of the sorted
// diagram).
//
// Adding a cell to row `r` appends a unique cell `(f, g)` at column
// `f = row[s, r]` and row-rank `g = freq[s, f]`. The two sorted diagrams are
// transposes (conjugates) iff the multiset of `(f, g)` on `Left` equals the
// multiset of transposed `(g, f)` on `Right`. Summing `splitmix64` over packed
// 64-bit coordinates hashes each multiset injectively with high probability in
// O(1) per update.


enum Side: Left | Right

@!multitest
let (n, q): (1..=200_000, 1..=200_000) = read()
domain Row = 1..=n
var row: [Side, Row]Int = 0
var freq: [Side, 0..q]Int = 0
var h: [Side]U64 = 0

fn add(s: Side, r: Row):
    let f = row[s, r]
    row[s, r] = f + 1
    let g = freq[s, f]
    freq[s, f] = g + 1
    let key = match s:
        Left => (U64(f) << 32) | U64(g)
        Right => (U64(g) << 32) | U64(f)
    h[s] += splitmix64(key)

for _ in 0..q:
    let (r0, r1): (Row, Row) = read()
    add(Left, r0)
    add(Right, r1)
    print(if h[Left] == h[Right]: "YES" else: "NO")
