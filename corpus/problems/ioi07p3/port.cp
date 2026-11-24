// Source: IOI 2007 P3 (Sails), https://dmoj.ca/problem/ioi07p3
// limits: 600 ms, 64 MB
// Problem: N <= 1e5 masts, mast i of height H_i <= 1e5 with K_i <= H_i sails, at most one per
//   unit height. A sail costs the number of sails behind it at its height. Print the least total.
// Solution: only the count c at each height matters (total sum c(c - 1) / 2), not the mast order.
//   Taking masts by height, each greedily puts its K sails on the K least-used heights in 1..=H.
//   Counts never rise with height, so with v the count at H - K + 1 those are every height up to
//   H whose count is below v, plus the lowest heights whose count is v: adding there keeps the
//   order. A range-add min SegTree finds both runs with `right_while`, O(N log H).
// original: corpus/submissions/5120005.cpp
// Bounds: n: 2..=1e5; masts.h, masts.k: 1..=1e5
let n: 2..=1e5 = read()
let masts: [n](h: 1..=1e5, k: 1..=1e5) = read()
domain Height = 1..=1e5
var sails = SegTree<Height, Int, Int>(min, act: add, a: [Height] _ => 0)

for (h, k) in masts.sorted():
    let top = Height(h).r
    let v = sails.get(Height(h - k + 1))
    let level = sails.right_while(Height.cuts.first!, c => c > v)  // the first height at v
    let below = min(sails.right_while(level, c => c >= v), top)    // the first height under v
    sails.apply(below..top, 1)
    let rest = k - (Int(top) - Int(below))
    sails.apply(level..Height.cuts(Int(level) + rest), 1)

print(Height.map(x => sails.get(x)).map(c => c * (c - 1) / 2).sum())
