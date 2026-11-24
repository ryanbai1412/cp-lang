// Source: Codeforces 2201F1 (Monotone Monochrome Matrices, Medium Version),
//   https://codeforces.com/contest/2201/problem/F1
// limits: 4000 ms, 512 MB
// Problem: an n x n white matrix; q queries each blacken a white cell (r, c), then ask whether
//   it is monotone: no rows i < j and columns k < l with C[i,k] = C[j,l] != C[j,k] = C[i,l].
//   Print YES/NO each time. n, q <= 2e5 (summed over tests). Multitest.
// Solution: no such 2x2 pattern means the matrix is the only one with its row and column
//   counts, which holds iff the sorted column counts are the conjugate of the sorted row
//   counts. Keep both as Ferrers diagrams: a new cell in row r adds cell (f, g) with f its old
//   count and g the number of rows already that long, and the column side the same,
//   transposed. Compare the two cell multisets by summed splitmix64 hashes: O(1) per query.
// Bounds: n, q: 1..=200_000; r0, r1: 1..=n


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
    print(yes(h[Left] == h[Right]))
