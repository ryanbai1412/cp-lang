// Source: IOI 2007 P3 (Sails), https://dmoj.ca/problem/ioi07p3
// Problem: N <= 1e5 masts, mast i of height H_i <= 1e5 with K_i <= H_i sails, at most one per
//   unit height. A sail costs the number of sails behind it at its height. Print the least total.
// Solution: only the count c at each height matters (total sum c(c - 1) / 2), not the mast order.
//   Taking masts by height, each greedily puts its K sails on the K least-used heights in 1..=H.
//   Counts stay non-increasing in height, so those are the heights below v = count[H - K + 1] up
//   to H, plus the leftmost end of the block equal to v; adding there keeps the order. A range-add
//   segment tree with binary searches, O(N log^2 H).
// Runnable port (proto/cp.py lib-run) of statement-first ioi07p3.cp
// original: corpus/submissions/5120005.cpp

let n: Int = read()
domain M = 0..n
let masts: [M](h: Int, k: Int) = read()

var max_h = 0
var pairs: List<(h: Int, k: Int)> = []
for m in M:
    max_h max= masts[m].h
    pairs.push(masts[m])
pairs = pairs.sorted()

domain H = 1..=max_h

let init = [H] _ => 0
var st = SegTree<H, Int, Int>(min, act: add, a: init)

// First height in 1..=limit with count <= target (non-increasing array).
fn first_le(mut st: SegTree<H, Int, Int>, limit: Int, target: Int) -> Int:
    let lo = (1..limit + 1).first_true(mid => st.get(H(mid)) <= target) ?? (limit + 1)
    lo

for p in pairs:
    let h = p.h
    let k = p.k
    let lo_idx = h - k + 1
    let v = st.get(H(lo_idx))
    // Leftmost index in 1..=h with count <= v, and leftmost with count <= v - 1.
    let left_eq = first_le(mut st, h, v)
    let right_after = first_le(mut st, h, v - 1)
    if right_after <= h:
        st.apply(H(right_after)..=H(h), 1)
    let rem = k - (h - right_after + 1)
    if rem > 0:
        st.apply(H(left_eq)..=H(left_eq + rem - 1), 1)

var ans = 0
for i in H:
    let c = st.get(i)
    ans += c * (c - 1) / 2
print(ans)
