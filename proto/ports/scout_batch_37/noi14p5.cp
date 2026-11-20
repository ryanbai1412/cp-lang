// NOI 2014 Day 2 P2, Random Number Generator
// original: corpus/submissions/3601452.cpp
let x0: 0..=100_000_000 = read()
let a: 0..=300 = read()
let b: 0..=100_000_000 = read()
let c: 0..=100_000_000 = read()
let d: 1..=100_000_000 = read()
let n: 1..=5000 = read()
let m: 1..=5000 = read()
let q: 0..=50_000 = read()
domain Cell = 1..=n * m

// the board in row-major order holds a shuffle of 1..=n * m
var board: [Cell]Int = [Cell] i => i.to_idx + 1
var x = x0
for i in Cell:
    x = (a * x * x + b * x + c) % d
    swap(mut board[i], mut board[Cell.from_idx(x % (i.to_idx + 1))!])
for _ in 0..q:
    let (u, v): (Cell, Cell) = read()
    swap(mut board[u], mut board[v])
var at: [1..=n * m]Cell = Cell.first
for i in Cell: at[board[i]] = i

// A set of cells lies on one right/down path when no two of them are
// anti-ordered. lo[r]..=hi[r] are the columns of row r still compatible
// with every cell taken; take the values in increasing order when they fit.
var lo: [0..n]Int = 0
var hi: [0..n]Int = m - 1
var path: List<Int> = []
for v in 1..=n * m:
    let (r, col) = (at[v].to_idx / m, at[v].to_idx % m)
    if col in lo[r]..=hi[r]:
        path.push(v)
        for above in 0..r: hi[above] min= col
        for below in r + 1..n: lo[below] max= col
print(path)
