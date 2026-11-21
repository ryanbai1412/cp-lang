// Codeforces 2129 D: Permutation Blackhole.
// Coloring cells `0..n` in a uniformly random permutation charges +1 to the
// nearest already-colored neighbor of each newly colored cell (ties go left).
// Because every charge to a cell halves its remaining distance to the next
// charge on that side, no cell can receive more than `MAX_SCORE = 10` charges
// for `n <= 100`.
// `dp[i, j + 1, vl, vr]` is the probability that coloring `i..=j` (bounded by
// already-colored `i - 1` and `j + 1`) delivers `vl` charges to `i - 1` and
// `vr` charges to `j + 1` (tracked as 0 when that boundary is unconstrained)
// while satisfying all constraints inside `i..=j`. Multiplying by `n!` converts
// probability back to permutation count.
const MAX_SCORE = 10
type Z = Mod<998_244_353>
let comb = Comb<Z>()

@!multitest
let n: 1..=100 = read()
let target = [0..n] _ => read<?Int>(none: -1)

var dp: [0..=n, 0..=n, 0..=MAX_SCORE, 0..=MAX_SCORE]Z = 0
for i in 0..=n: dp[i, i, 0, 0] = 1

for len in 1..=n:
    let pick_prob = Z(len).inv
    for i in 0..=n - len:
        let j = i + len - 1
        let lim_l = if i > 0: target[i - 1] else: none
        let lim_r = if j + 1 < n: target[j + 1] else: none
        let vl_cap = if let c = lim_l: min(c, MAX_SCORE) else: 0
        let vr_cap = if let c = lim_r: min(c, MAX_SCORE) else: 0
        for k in i..=j:
            // Nearest boundary to `k` (ties break left).
            let add_l = if lim_l != none and (j + 1 == n or k - i <= j - k): 1 else: 0
            let add_r = if lim_r != none and (i == 0 or j - k < k - i): 1 else: 0
            let lim_k = target[k]
            let vkl_cap = if let c = lim_k: min(c, MAX_SCORE) else: 0
            for vl in 0..=vl_cap:
                for vkl in 0..=vkl_cap:
                    let left = dp[i, k, vl, vkl]
                    if left == 0: continue
                    let vkr = if let c = lim_k: c - vkl else: 0
                    if vkr > MAX_SCORE: continue
                    let lw = left * pick_prob
                    for vr in 0..=vr_cap:
                        let right = dp[k + 1, j + 1, vkr, vr]
                        if right != 0:
                            dp[i, j + 1, vl + add_l, vr + add_r] += lw * right

print(dp[0, n, 0, 0] * comb.fact(n))
