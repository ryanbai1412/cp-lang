// Source: Baltic OI 2004 Day 1 P3 (Sequence), https://dmoj.ca/problem/btoi04p3
// limits: 2000 ms, 512 MB
// Problem: given t_1..t_N (N <= 1e6, 0 <= t_i <= 2e9), find a strictly increasing integer
//   z_1 < ... < z_N (|z_i| <= 2e9) minimizing sum |t_i - z_i|; print that minimum, then
//   any optimal z, one per line.
// Solution: with x_i = t_i - i and y_i = z_i - i the order becomes non-decreasing: L1
//   isotonic regression. Pool adjacent violators: keep a stack of blocks, each fitted by
//   its median; merge the new block into the top while the top's median is larger. Each
//   block keeps its smallest ceil(len / 2) values, negated, in a min heap of a MeldHeaps pool
//   (top = -median), so a merge melds the heaps and pops down to size: O(N log N); z_i = median + i.
// original: corpus/submissions/5270736.cpp
// Bounds: n: 1..=1e6; t: 0..=2e9
use std.heap.{Meld, MeldHeaps}

let n: 1..=1e6 = read()
domain V = 0..n
let t: [V]0..=2e9 = read()
fn x(i: V) -> Int = t[i] - i.idx
type Key = -2_000_000_000..=1_000_000  // -x

struct Block(from: V, to: V, len: Int, low: Meld<Key>)
var pool = MeldHeaps<Key>()
pool.reserve(n)
var blocks = List<Block>()
for i in V:
    let low: Key = -x(i)
    let one: List<Key> = [low]
    var b = Block(i, i, 1, pool.of(one))
    while let top = blocks.last:
        if pool.top(top.low)! >= pool.top(b.low)!:
            break
        blocks.pop()
        b = Block(top.from, b.to, top.len + b.len, pool.meld(top.low, b.low))
        while b.low.len > (b.len + 1) / 2:
            pool.pop(mut b.low)
    blocks.push(b)

var cost = 0
for b in blocks:
    for i in b.from..=b.to: cost += abs(x(i) + pool.top(b.low)!)
print(cost)
for b in blocks:
    for i in b.from..=b.to: print(i.idx - pool.top(b.low)!)
