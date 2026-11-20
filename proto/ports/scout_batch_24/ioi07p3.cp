// IOI '07 P3 - Sails (DMOJ: ioi07p3, 25pt, TL 0.6s, ML 64MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// N <= 1e5 masts; mast i has height H_i <= 1e5 and K_i <= H_i sails.
// Minimize sum_h C(c_h, 2) where c_h is the number of sails at height h.
//
// Sort masts by H_i ascending. Maintain row sail counts in non-increasing order
// c[1] >= c[2] >= ... >= c[max_h]. Placing K_i sails on the K_i smallest counts
// in 1..=H_i preserves sorted order if we split the tie block at value v = c[H_i - K_i + 1]:
// increment all positions strictly < v in 1..=H_i, and increment the leftmost
// remaining positions inside the == v block.
use std.algebra.{Monoid, Action}
use std.lazy.{LazySegTree}

let n: 1..=1e5 = read()
domain M = 0..n
struct Mast(h: 1..=1e5, k: 1..=1e5)
    where k <= h
let masts_in: [M]Mast = read()

// WISH A1: `MinOpt` and `AddMin` are missing from `stdlib/lazy.cp` even though
// `MaxOpt` and `AddMax` are provided; every min-based lazy tree rewrites them.
const MinOpt = Monoid<?Int>(
    id: none,
    op: (a, b) => if a == none: b else: if b == none: a else: min(a!, b!)
)
const AddMin = Action<?Int, Int>(
    id: 0,
    compose: (f, g) => f + g,
    apply: (f, x) => if x == none: none else: x! + f
)

fn solve() -> Int:
    var masts = [masts_in[i] for i in M]
    sort(mut masts, by: m => m.h)
    let max_h = masts.last!.h
    domain H = 1..=max_h

    // Counts kept non-increasing: fold(1..=p) is min(c[1..=p]) == c[p].
    var st = LazySegTree<H, ?Int, Int>.from(MinOpt, AddMin, [H] _ => 0)

    for m in masts:
        let pivot = H(m.h - m.k + 1)
        let v = st.get(pivot)!
        // WISH A2: `LazySegTree` lacks `max_right` and `min_left`, forcing either
        // O(log^2 n) binary search over `get()` or a hand-rolled lazy tree.
        let left_end = st.max_right(H.first.l, x => x == none or x! > v)
        let right_end = st.max_right(H.first.l, x => x == none or x! >= v)
        let right_bound = min(right_end, H(m.h).r)

        // Strict tail (< v) inside 1..=m.h:
        if right_bound < H(m.h).r:
            st.apply(right_bound..H(m.h).r, 1)

        // Leftmost remaining increments inside the == v block:
        let tail_cnt = H(m.h).r.ord - right_bound.ord
        let rem = m.k - tail_cnt
        if rem > 0:
            // WISH A3: advancing a `Bounds<H>` by an integer offset `rem` has no
            // ergonomic helper; `H.bounds.nth(left_end.ord + rem)` drops to raw `.ord`.
            let tie_hi = H.bounds.nth(left_end.ord + rem)
            st.apply(left_end..tie_hi, 1)

    var ans = 0
    for h in H:
        let c = st.get(h)!
        ans += c * (c - 1) / 2
    ans

@brute(solve)
fn solve_brute() -> Int
    where n <= 50:
    var masts = [masts_in[i] for i in M]
    sort(mut masts, by: m => m.h)
    let max_h = masts.last!.h
    var c = List.filled(max_h + 1, 0)
    for m in masts:
        var idxs = [i for i in 1..=m.h]
        sort(mut idxs, by: i => c[i])
        for j in 0..m.k:
            c[idxs[j]] += 1
    sum(x * (x - 1) / 2 for x in c)

print(solve())
