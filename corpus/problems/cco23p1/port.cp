// Source: CCO 2023 Day 1 P1 (Binaria), https://dmoj.ca/problem/cco23p1
// limits: 1000 ms, 1024 MB
// Problem: given the sums of all n - k + 1 windows of k consecutive bits of a binary string of
//   length n <= 1e6 (1 <= k <= n, at least one string fits), count the strings that fit,
//   modulo 1e6 + 3.
// Solution: window w + 1 minus window w is bit[w + k] - bit[w], so each residue class mod k
//   is a chain whose bits follow known offsets from its first bit. Offsets spanning more than
//   one value: no string; spanning two: forced (first bit 1 iff some offset is -1); else free.
//   The first window then says how many free chains are 1: C(free, total[0] - forced). O(n).
// original: corpus/submissions/5579927.cpp
// Bounds: n: 1..=1e6; k: 1..=n; total: 0..=k
type Z = Mod<1_000_003>
let (n, k): (1..=1e6, 1..=n) = read()
domain Window = 0..n - k + 1
let total: [Window]0..=k = read()

fn ways() -> Z:
    var free = 0
    var ones = 0
    for r in 0..k:
        let steps = [total[v] - total[w] for w in Window.drop(r).step(k) if let v = w.next]
        let off = steps.scan(+)
        let seen = (0..=steps.len).map(i => off.prefix(i))
        let (lo, hi) = (seen.min()!, seen.max()!)
        if hi - lo > 1: return Z(0)
        if hi == lo: free += 1
        elif lo == -1: ones += 1
    let need = total[Window.first!] - ones
    if need < 0 or need > free: return Z(0)
    Comb<Z>(free).choose(free, need)

print(ways())
