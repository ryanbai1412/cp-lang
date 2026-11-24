// Source: Codeforces 2129 D (Permutation Blackhole), https://codeforces.com/contest/2129/problem/D
// limits: 3000 ms, 512 MB
// Problem: cells 1..=n turn black in the order of a permutation p; each newly coloured cell
//   (after the first) adds 1 to the score of its nearest black cell, ties to the left. Some
//   final scores s_i are fixed (-1 = free); count permutations matching them modulo
//   998244353. Multitest, n <= 100, sum of n^2 <= 1e4.
// Solution: each charge to a black cell halves the gap on that side, so a score is at most
//   bit_width(n) + 1. dp[l, r, vl, vr] is the probability that colouring the gap l..r (between
//   black cells l - 1 and r) charges vl to l - 1 and vr to r and meets every fixed score
//   inside: the first cell k is uniform, charges its nearer side, and splits the gap, with
//   s_k shared between its two halves. Multiply by n!. O(n^3 log^3 n).
// Bounds: n: 1..=100

type Z = Mod<998_244_353>
let comb = Comb<Z>()

enum Side: Left | Right | None

@!multitest
let n: 1..=100 = read()
let target = [0..n] _ => read<?Int>(none: -1)
let max_score = bit_width(n) + 1

fn cap(lim: ?Int) -> Int = if let c = lim: min(c, max_score) else: 0

fn nearest_boundary(l: Int, r: Int, k: Int) -> Side =
    if l == 0 and r == n: Side.None
    elif r == n: Side.Left
    elif l == 0: Side.Right
    elif k - l <= r - 1 - k: Side.Left
    else: Side.Right

var dp: [0..=n, 0..=n, 0..=max_score, 0..=max_score]Z = 0
for i in 0..=n: dp[i, i, 0, 0] = 1

for len in 1..=n:
    let pick_prob = Z(len).inv()
    for l in 0..=n - len:
        let r = l + len
        let lim_l = if l > 0: target[l - 1] else: None
        let lim_r = if r < n: target[r] else: None
        let vl_cap = cap(lim_l)
        let vr_cap = cap(lim_r)
        for k in l..r:
            let side = nearest_boundary(l, r, k)
            let add_l = Int(side == Side.Left and lim_l != None)
            let add_r = Int(side == Side.Right and lim_r != None)
            let lim_k = target[k]
            for vl in 0..=vl_cap:
                for vkl in 0..=cap(lim_k) if dp[l, k, vl, vkl] != 0:
                    let vkr = if let c = lim_k: c - vkl else: 0
                    if vkr <= max_score:
                        let lw = dp[l, k, vl, vkl] * pick_prob
                        for vr in 0..=vr_cap if dp[k + 1, r, vkr, vr] != 0:
                            dp[l, r, vl + add_l, vr + add_r] += lw * dp[k + 1, r, vkr, vr]

print(dp[0, n, 0, 0] * comb.fact(n))
