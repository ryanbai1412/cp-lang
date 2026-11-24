// Source: Baltic OI 2019 P2 (Nautilus), https://dmoj.ca/problem/btoi19p2
// limits: 1000 ms, 256 MB
// Problem: an R x C map (R, C <= 500) of water '.' and island '#', and M <= 5000 signals, each
//   N, E, S, W or '?' (any of the four). A submarine started on some water cell and made one move
//   per signal, always on water. Print how many cells it can be in now.
// Solution: keep each row's possible cells as a bitset: a signal ORs in the row above or below,
//   or the row shifted a column either way (all four for '?'), then masks it with the row's
//   water. Count the bits at the end. O(R C M / 64).
// original: corpus/submissions/4456161.cpp
// Bounds: r, c: 1..=500; m: 1..=5000
let r: 1..=500 = read()
let c: 1..=500 = read()
let m: 1..=5000 = read()
domain Row = 0..r
domain Col = 0..c
type Cells = [Row]Bitset<Col>
let map: [Row][Col]Char = read()
let signals: [m]Char = read()

let sea: Cells = [Row] i => Bitset<Col>.from([Col] j => map[i][j] == '.')

fn heard(sig: Char, d: Char) -> Bool = sig == d or sig == '?'

fn step(at: Cells, sig: Char) -> Cells = [Row] i => after(at, sig, i)

fn after(at: Cells, sig: Char, i: Row) -> Bitset<Col>:
    var x = Bitset<Col>.new()
    if heard(sig, 'N'): if let below = i.next: x.union(at[below])
    if heard(sig, 'S'): if let above = i.prev: x.union(at[above])
    if heard(sig, 'E'): x.union_shifted(at[i], 1)
    if heard(sig, 'W'): x.union_shifted(at[i], -1)
    x.intersect(sea[i])
    x

let end = signals.fold_from(sea, step)
print(sum(end[i].len for i in Row))
