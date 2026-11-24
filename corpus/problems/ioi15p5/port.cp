// Source: IOI 2015 Day 2 P2 (Sorting), https://dmoj.ca/problem/ioi15p5
// limits: 4000 ms, 256 MB
// Problem: a permutation S of 0..n-1 (n <= 2e5) and Ermek's m <= 6e5 planned swaps (X[i], Y[i]).
//   Each round Ermek makes his next swap, then Aizhan swaps any two positions; she stops once S
//   is sorted. Print the least number of rounds R that sorts S, then Aizhan's R swaps.
// Solution: after Ermek's first r swaps his row needs n - cycles swaps, and swaps chosen as pairs
//   of values commute with his, so R is the least r with n - cycles(r) <= r: a binary search on
//   r, O(n + r) each. Plan the value swaps that sort the row Ermek alone leaves, then replay the
//   rounds: in round t swap wherever those two values now sit (`0 0` once sorted). O((n + m) log m).
// original: corpus/submissions/5723358.cpp (grader function findSwapPairs)
// stdin here, as the IOI sample grader: n, the permutation, m, then Ermek's m swaps.
// Bounds: n: 1..=200_000; start, ermek.x, ermek.y: 0..=n - 1; m: 1..=600_000
use std.seq.{perm_cycles}

let n: 1..=200_000 = read()
domain Pos = 0..n
let start: [Pos]Pos = read()
let m: 1..=600_000 = read()
let ermek: [m](x: Pos, y: Pos) = read()

fn after(r: Int) -> [Pos]Pos:
    var a = start.clone()
    for e in ermek.take(r): a.swap_at(e.x, e.y)
    a

// a row and its inverse `at` (where each value sits), swapped together
struct Row(val: [Pos]Pos, at: [Pos]Pos)
fn row(a: [Pos]Pos) -> Row = Row(a.clone(), a.inverse())
fn swap(mut w: Row, x: Pos, y: Pos):
    w.val.swap_at(x, y)
    w.at[w.val[x]] = x
    w.at[w.val[y]] = y

let rounds = (0..=m).first_true(r => n - perm_cycles(after(r)).len <= r)!

// the value swaps that sort Ermek's final row: put value i at position i, in turn
var end = row(after(rounds))
var plan: List<(Pos, Pos)> = []
for i in Pos:
    let v = end.val[i]
    if v != i:
        plan.push((i, v))
        swap(mut end, i, end.at[i])

var now = row(start)
print(rounds)
for (t, e) in ermek.take(rounds).items:
    swap(mut now, e.x, e.y)
    let (u, v) = plan.get(t) ?? (now.val[Pos(0)], now.val[Pos(0)])
    let (pu, pv) = (now.at[u], now.at[v])
    print(pu, pv)
    swap(mut now, pu, pv)
