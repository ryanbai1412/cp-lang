// Source: IOI 2018 Day 1 P2 (Seats), https://dmoj.ca/problem/ioi18p2
// Problem: an h x w hall (h * w <= 1e6) seats contestants 0..h*w; a prefix 0..k is beautiful when
//   its seats form a rectangle. After each of q <= 5e4 swaps of two contestants' seats, print the
//   number of beautiful prefixes.
// Solution: a prefix is a rectangle iff it has exactly one corner (a cell whose upper and left
//   neighbours are outside) and no dent (an outside cell with 2+ inside neighbours). Each cell
//   adds 1 over the range of prefixes where it is a corner or a dent; a segment tree with range
//   add keeps the minimum and how many prefixes reach it; every prefix has a corner and the full
//   hall scores 1, so the minimum is 1 and its count is the answer.
//   A swap re-accounts the two seats and their neighbours: O(log(hw)) per swap.
// original: corpus/submissions/4731161.cpp (grader functions give_initial_chart / swap_seats)
// stdin: h w q, the h * w seats (row, column) of contestants 0.., then q lines a b.
// stdout: the number of beautiful prefixes after each swap.
// Bounds: h, w: 1..=1000; q: 0..=50_000; seat.r: 0..=h - 1; seat.c: 0..=w - 1
use std.algebra.{MinCount}

let h: 1..=1000 = read()
let w: 1..=1000 = read()
let q: 0..=50_000 = read()
assert h * w <= 1_000_000
domain Row = 0..h
domain Col = 0..w
domain Who = 0..h * w
var seat: [Who](r: Row, c: Col) = read()
var who: [Row, Col]Who = Who.first!
for p in Who: who[seat[p].r, seat[p].c] = p

// position t of the tree stands for the prefix 0..=t; it holds that prefix's top-left corners
// (a seated cell with nobody above or to its left) plus its dents (an empty cell with two or
// more seated neighbours). A prefix is a rectangle exactly when the total is 1, the least.
let shift = Action<MinCount<Int>, Int>.new(apply: (d, v, _) => MinCount(v.val + d, v.count), over: MinCount.monoid(), commutes: true)
var bad = SegTree<Who, MinCount<Int>, Int>(MinCount.monoid(), [Who] _ => MinCount.of(0), act: shift)

// the cut where the contestant at (r, c) sits down; off the grid, never
fn arrival(r: Int, c: Int) -> Cut<Who>:
    if let row = Row.at(r), let col = Col.at(c): who[row, col].l else: Who.cuts.last!

fn account(r: Row, c: Col, d: Int):
    let (y, x) = (Int(r), Int(c))
    let me = who[r, c].l
    // a corner from my arrival until the first of my upper and left neighbours
    let covered = min(arrival(y - 1, x), arrival(y, x - 1))
    if me < covered: bad.apply(me..covered, d)
    // a dent from my second neighbour's arrival until my own
    var around = [arrival(y - 1, x), arrival(y + 1, x), arrival(y, x - 1), arrival(y, x + 1)]
    around.sort()
    if around[1] < me: bad.apply(around[1]..me, d)

for r in Row:
    for c in Col: account(r, c, 1)

for (a, b) in read<[q](a: Who, b: Who)>():
    // the two seats and their neighbours are the only cells whose spans change
    var cells: List<(Row, Col)> = []
    for p in [a, b]:
        let (y, x) = (Int(seat[p].r), Int(seat[p].c))
        for (dy, dx) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)]:
            if let row = Row.at(y + dy), let col = Col.at(x + dx): cells.push((row, col))
    cells.sort()
    cells.dedup()
    for (r, c) in cells: account(r, c, -1)
    seat.swap_at(a, b)
    who[seat[a].r, seat[a].c] = a
    who[seat[b].r, seat[b].c] = b
    for (r, c) in cells: account(r, c, 1)
    print(bad.all.count)
