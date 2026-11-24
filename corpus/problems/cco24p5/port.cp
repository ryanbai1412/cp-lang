// Source: CCO 2024 Day 2 (Heavy Light Decomposition), https://dmoj.ca/problem/cco24p5
// limits: 4000 ms, 1024 MB
// Problem: an array a of N <= 5e5 values in 1..N. A value is heavy in an array if it occurs
//   more than once there, else light; an array is good if light and heavy elements alternate.
//   Count the ways to cut a into good subarrays, mod 1000003.
// Solution: dp[i] = sum of dp[j] over ends j > i with a[i..j) good, filled from the right. The
//   first element is light or heavy, so a good array follows one of two parity patterns, one
//   SegTree of MinCount each over ends j, with range add: it keeps the least number of violated
//   conditions and the sum of dp[j] where that least is reached. Prepending a[i] adds or removes violations on ranges
//   of ends from the next occurrences of its value: a light one must not repeat, a heavy one
//   needs a later copy at a heavy place. dp[i] adds the sums where 0 are violated. O(N log N).
// Bounds: n: 1..=5e5; a: 1..=n
// original: corpus/submissions/6435483.cpp
use std.algebra.{MinCount}
let mod = 1_000_003
let n: 1..=5e5 = read()
domain I = 0..n
domain End = 0..=n
domain Value = 1..=n
domain Side = 0..2
let a: [I]Value = read()

var dp: [End]Int = 0
dp[End.last!] = 1
let shift = Action<MinCount<Int>, Int>.new(apply: (d, v, _) => MinCount(v.val + d, v.count), over: MinCount.monoid(), commutes: true)
var bad = [Side] _ => SegTree<End, MinCount<Int>, Int>(MinCount.monoid(), fill: MinCount(0, 0), act: shift)
var next_at: [Side, Value]?I = None
var next_any: [Value]?I = None
var stop_of: [Side, I]End = End.last!

fn after(i: I) -> End = End(i.ord + 1)

for i in I.rev:
    let x = a[i]
    for s in Side:
        bad[s].update(after(i), c => MinCount(c.val, dp[after(i)]))
        if let u = next_at[Side(1 - s.ord), x]: bad[s].apply(after(u)..=End.last!, 1)
        if i.ord % 2 != s.ord:
            if let g = next_any[x]: bad[s].apply(after(g)..=End.last!, 1)
        else:
            var stop = End.last!
            if let d = next_at[s, x]:
                bad[s].apply(after(d)..=stop_of[s, d], -1)
                stop = End(d.ord)
            stop_of[s, i] = stop
            bad[s].apply(after(i)..=stop, 1)
        let best = bad[s].all
        if best.val == 0: dp[End(i.ord)] = (dp[End(i.ord)] + best.count) % mod
    next_at[Side(i.ord % 2), x] = Some(i)
    next_any[x] = Some(i)

print(dp[End.first!])
