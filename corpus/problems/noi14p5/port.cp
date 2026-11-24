// Source: NOI 2014 Day 2 P2 (Random Number Generator), https://dmoj.ca/problem/noi14p5
// limits: 2000 ms, 256 MB
// Problem: x[i] = (a x[i-1]^2 + b x[i-1] + c) mod d drives NM swaps (the i-th exchanges T[i]
//   and T[x[i] mod i + 1]) of 1..NM, then Q given swaps; T fills an N x M grid row by row.
//   Print the lexicographically smallest sorted list of values on a right/down path from the
//   top-left to the bottom-right. N, M <= 5000, Q <= 50000.
// Solution: build the permutation (the i-th swap is an inside-out shuffle step) and each
//   value's cell. Cells share a right/down path iff no two are anti-ordered, so take values
//   in increasing order whenever their column lies in the row's still-allowed range
//   lo[r]..=hi[r]; a taken cell caps hi of rows above and raises lo of rows below.
//   O(NM + (N + M) N).
// original: corpus/submissions/3601452.cpp
// Bounds: x0, b, c: 0..=100_000_000; a: 0..=300; d: 1..=100_000_000; n, m: 1..=5000;
//   q: 0..=50_000; u, v: 1..=cells
let x0: 0..=100_000_000 = read()
let a: 0..=300 = read()
let b: 0..=100_000_000 = read()
let c: 0..=100_000_000 = read()
let d: 1..=100_000_000 = read()
let n: 1..=5000 = read()
let m: 1..=5000 = read()
let q: 0..=50_000 = read()
let cells = n * m
// a cell index, stored in 32 bits: two NM-cell boards at 8 bytes each would not fit 256 MB
type Cell = 0..25_000_000

// the board in row-major order holds a shuffle of 0..cells
var board: [0..cells]Cell = 0
var x = x0
for i in 0..cells:
    x = (a * x * x + b * x + c) % d
    let j = x % (i + 1)
    board[i] = board[j]
    board[j] = i
for _ in 0..q:
    let u: 1..=cells = read()
    let v: 1..=cells = read()
    board.swap(u - 1, v - 1)
var at: [0..cells]Cell = 0
for i in 0..cells: at[board[i]] = i

// A set of cells lies on one right/down path when no two of them are
// anti-ordered. lo[r]..=hi[r] are the columns of row r still compatible
// with every cell taken; take the values in increasing order when they fit.
var lo: [0..n]Int = 0
var hi: [0..n]Int = m - 1
var path: List<Int> = []
for v in 0..cells:
    let r = at[v] / m
    let col = at[v] % m
    if lo[r] <= col <= hi[r]:
        path.push(v + 1)
        for above in 0..r: hi[above] min= col
        for below in (r + 1)..n: lo[below] max= col
print(path)
