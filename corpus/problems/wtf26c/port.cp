// Source: AtCoder World Tour Finals 2026 C (Count by Descents),
//   https://atcoder.jp/contests/awtf2026algo/tasks/awtf2026algo_c
// limits: 10000 ms, 2048 MB
// Problem: given strictly increasing A_1 < ... < A_N (N <= 80, A_i < 998244353), for each
//   d = 0..N-1 sum prod (i + x_i) over the rearrangements x of A with exactly d descents,
//   mod 998244353.
// Solution: labelled[g] counts the arrangements whose groups take labels from a set of g:
//   dp[i, j] over the sorted A, i closed and j open slots; each step keeps a slot (weight
//   cur + 1 - i or j + 1) or, weighted A + 1, joins the claim table
//   claim[i, j] = C(i + j, i) * sum_{k < g} k^i (k + 1)^j by a 2D convolution, flattened into one
//   NTT. Two binomial inversions, exact[h] = labelled[h] - sum_{g < h} C(h, g) exact[g] and
//   ans[g] = exact[g] - sum_{h < g} C(n - h, g - h) ans[h], give d = g - 1. O(N^4 log N).

type Z = Mod<998_244_353>
let comb = Comb<Z>()

let n: Int = read()
let a: [0..n]Int = read()

// The product of `x` and `y` on rows 0..rows and columns 0..cols, through one
// 1D convolution: row stride 2 * rows - 1 keeps row sums from wrapping.
fn convolve_2d(x: [0..=n, 0..=n]Z, y: [0..=n, 0..=n]Z, rows: Int, cols: Int) -> [0..=n, 0..=n]Z:
    let stride = 2 * rows - 1
    fn flat(m: [0..=n, 0..=n]Z) -> List<Z> =
        [if t % stride < rows: m[t % stride, t / stride] else: Z(0) for t in 0..stride * cols]
    let prod = convolve(flat(x), flat(y))
    [0..=n, 0..=n] (i, j) => if i < rows and j < cols: prod[i + stride * j] else: Z(0)

var labelled: [0..=n]Z = 0
var power_sum: [0..=n, 0..=n]Z = 0
for g in 1..=n:
    for i in 0..n, j in 0..n - i: power_sum[i, j] += Z(g - 1) ** i * Z(g) ** j
    let claim = [0..=n, 0..=n] (i, j) => power_sum[i, j] * comb.choose(i + j, i)

    var dp: [0..=n, 0..=n]Z = 0
    dp[0, 0] = 1
    for cur in 0..n:
        let (rows, cols) = (cur + 1, n - cur)
        let joined = convolve_2d(claim, dp, rows, cols)
        var next: [0..=n, 0..=n]Z = 0
        for i in 0..rows, j in 0..cols:
            next[i, j] += dp[i, j] * (cur + 1 - i)
            next[i + 1, j] += dp[i, j + 1] * (j + 1) + joined[i, j] * (a[cur] + 1)
        dp = next
    labelled[g] = dp[n, 0]

var ans = labelled
for h in 1..=n, g in 1..h: ans[h] -= comb.choose(h, g) * ans[g]
for g in 1..=n, h in 1..g: ans[g] -= comb.choose(n - h, g - h) * ans[h]

print(ans[1..=n])
