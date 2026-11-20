// CCO 2023 P1 Binaria: count binary strings of length n whose every window of
// k consecutive bits has the given sum, modulo 1e6 + 3.
// Neighbouring windows differ by bit[i + k] - bit[i], so each residue class
// mod k is a chain of "same", "0 then 1" and "1 then 0" links. A chain with
// any unequal link is forced; the rest are free, and the first window says
// how many free chains must be 1.
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
    let need = total[Window.first] - forced_ones
    if need < 0 or need > free: return 0
    choose(free, need)

print(ways(total))
