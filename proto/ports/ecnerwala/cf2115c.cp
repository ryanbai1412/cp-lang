// Codeforces 2115 C: Gellyfish and Eternal Violet.
// To reduce all `n` monsters to HP 1 in `m` rounds (each round shines with
// probability `p`, hitting all monsters for -1, or misses with probability `1 - p`,
// hitting one chosen monster for -1), decompose required damage into:
//   - `tot_all = min_h - 1` global -1 layers, and
//   - `tot_ones = sum(h[i] - min_h)` excess single-target hits.
// Solve in three phases:
//   1. `endgames[k]` (`tot_ones == 0`): DP over `(i, j)` = `i` global layers left and
//      `j` monsters still unchipped on the current layer; on a non-shine at `j == 0`
//      and `i > 0`, either start chipping a new layer (`dp[i - 1, n - 1]`) or skip
//      (`dp[i, 0]`), taking the max.
//   2. `endgames2[k]` (`tot_all == 0`): shines are wasted, non-shines reduce `ones`.
//   3. `win[ones, alls]`: while both > 0, shine drops `alls` and non-shine drops `ones`,
//      anchored on `endgames` and `endgames2`.
@!multitest
let n: 1..=20 = read()
let m: 1..=4_000 = read()
let p100: 0..=100 = read()
let p = F64(p100) / 100.0
let h: [0..n]1..=400 = read()

let min_h = h.min()!
let tot_ones = h.sum() - n * min_h
let tot_all = min_h - 1
let needed = tot_ones + tot_all
if needed > m:
    print(0.0)
    return

fn capture_idx(step: Int, floor: Int) -> ?Int:
    let rem = m - step
    if rem in floor..=needed: some(needed - rem) else: none

var endgames: [0..=tot_all]F64 = 0.0
var dp: [0..=tot_all, 0..n]F64 = 0.0
var ndp: [0..=tot_all, 0..n]F64 = 0.0
dp[0, 0] = 1.0
for step in 0..=m:
    if let idx = capture_idx(step, tot_ones): endgames[idx] = dp[idx, 0]
    for i in 0..=tot_all:
        for j in 0..n:
            let on_shine = if i > 0: dp[i - 1, j] else: dp[0, j]
            let on_miss = if j > 0:
                dp[i, j - 1]
            elif i > 0:
                max(dp[i - 1, n - 1], dp[i, 0])
            else:
                1.0
            ndp[i, j] = p * on_shine + (1.0 - p) * on_miss
    swap(mut dp, mut ndp)

var endgames2: [0..=tot_ones]F64 = 0.0
var dp2: [0..=tot_ones]F64 = 0.0
var ndp2: [0..=tot_ones]F64 = 0.0
dp2[0] = 1.0
for step in 0..=m:
    if let idx = capture_idx(step, tot_all): endgames2[idx] = dp2[idx]
    for i in 0..=tot_ones:
        let on_miss = if i > 0: dp2[i - 1] else: dp2[0]
        ndp2[i] = p * dp2[i] + (1.0 - p) * on_miss
    swap(mut dp2, mut ndp2)

var win: [0..=tot_ones, 0..=tot_all]F64 = 0.0
for j in 0..=tot_all: win[0, j] = endgames[j]
for i in 0..=tot_ones: win[i, 0] = endgames2[i]
for i in 1..=tot_ones:
    for j in 1..=tot_all:
        win[i, j] = p * win[i, j - 1] + (1.0 - p) * win[i - 1, j]
print(win[tot_ones, tot_all])
