// Source: CCO 2023 Day 1 P1 (Binaria), https://dmoj.ca/problem/cco23p1
// Problem: given the sums of all n - k + 1 windows of k consecutive bits of a binary string of
//   length n <= 1e6 (1 <= k <= n, at least one string fits), count the strings that fit,
//   modulo 1e6 + 3.
// Solution: window i + 1 minus window i is bit[i + k] - bit[i], so each residue class mod k
//   is a chain whose bits follow known offsets from its first bit. Offsets spanning more than
//   one value: no string; spanning two: forced (first bit 1 iff some offset is -1); else free.
//   The first window then says how many free chains are 1: C(free, total[0] - forced). O(n).
// original: corpus/submissions/5579927.cpp
use std.math.{choose}
const P = 1000003
type Z = Mod<P>
let n: 1..=1e6 = read()
let k: 1..=1e6 = read()
assert k <= n
domain Window = 0..n - k + 1
let total: [Window]0..=1e6 = read()

fn ways(total: [Window]0..=1e6) -> Z:
    var free = 0
    var forced_ones = 0
    for r in 0..k:
        // Offsets of the chain's bits relative to its first bit.
        var off = 0
        var lo = 0
        var hi = 0
        var w = r
        while w + 1 < Window.len:
            off += total[Window(w + 1)] - total[Window(w)]
            lo min= off
            hi max= off
            w += k
        if hi - lo > 1: return 0
        if hi == lo: free += 1
        elif lo == -1: forced_ones += 1
    let need = total[Window.first!] - forced_ones
    if need < 0 or need > free: return 0
    choose(free, need)

print(ways(total))
