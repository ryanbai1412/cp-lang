// Codeforces 2201 F2 — Conjugate Row-Count Multiset Hashing
//
// Maintain two Ferrers diagrams (`Left` and `Right`), where `row[s, r]` counts
// how many cells have been added to row `r` on side `s`, and `freq[s, f]`
// counts how many rows on side `s` currently have at least `f + 1` cells (i.e.
// the `(f + 1)`-th column length of the sorted Ferrers diagram).
//
// Adding a cell to row `r` appends a unique cell `(f, g)` to the sorted Ferrers
// diagram at column `f = row[s, r]` and row-rank `g = freq[s, f]`. The two
// sorted partitions are conjugates (transposes) of each other iff the multiset
// of cells `(f, g)` on `Left` equals the multiset of transposed cells `(g, f)`
// on `Right`, checked in O(1) per query via additive `splitmix64` multiset
// hashing over the packed 64-bit coordinates.

use std.math.{splitmix64}

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
