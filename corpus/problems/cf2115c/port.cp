// Source: Codeforces 2115C (Eternal Violet), https://codeforces.com/contest/2115/problem/C
// limits: 2000 ms, 1024 MB
// Problem: n <= 20 monsters with hp h_i <= 400 and m <= 4000 rounds. Each round the sword shines
//   with chance p, seen before acting; attacking hits every monster for 1 if it shines, else one
//   chosen monster. Print the best chance to bring every hp to exactly 1. Multitest, sum n <= 100.
// Solution: track (s, k): s = min(h) - 1 shines still needed, k = sum(h) - n min(h) single hits
//   that level the rest. While s, k > 0 every round makes progress, so rounds left is
//   slack + s + k (slack = m - s0 - k0) and win(k, s) is a plain memo recurrence. It exits at
//   k = 0, where a miss opens a new layer (s - 1, n - 1) or waits for a shine, or at s = 0, where
//   only k misses are needed: two DPs over rounds left, read off on their diagonals, cost
//   O(m (s0 n + k0)).
@!multitest
let n: 1..=20 = read()
let m: 1..=4_000 = read()
let p100: 0..=100 = read()
let p = F64(p100) / 100.0
let hp: [0..n]1..=400 = read()

let shines = hp.min()! - 1
let singles = hp.sum() - n * hp.min()!
let slack = m - shines - singles
if slack < 0: return print(0.0)

// Both edges of the (k, s) region are diagonals of a DP over rounds left:
// the chance from entry i is read after slack + i rounds.
fn diagonal<T>(size: Int, var dp: T, step: T -> T, at: (T, Int) -> F64) -> List<F64>:
    var out: List<F64> = []
    for r in 0..=slack + size:
        if r >= slack: out.push(at(dp, r - slack))
        dp = step(dp)
    out

// lv[s, k]: chance from (s, k < n) with r rounds left. A miss at k = 0 either
// starts a new layer, (s - 1, n - 1), or waits for a shine, (s, 0).
fn round(lv: [0..=shines, 0..n]F64) -> [0..=shines, 0..n]F64 =
    [0..=shines, 0..n] (s, k) => p * lv[max(s - 1, 0), k] + (1.0 - p) * miss(lv, s, k)
fn miss(lv: [0..=shines, 0..n]F64, s: 0..=shines, k: 0..n) -> F64 =
    if k > 0: lv[s, k - 1]
    elif s > 0: max(lv[s - 1, n - 1], lv[s, 0])
    else: 1.0
var lv0: [0..=shines, 0..n]F64 = 0.0
lv0[0, 0] = 1.0
let level = diagonal(shines, lv0, round, (lv, s) => lv[s, 0])

// fl[k]: with no shine needed, the chance of k misses in r rounds.
var fl0: [0..=singles]F64 = 0.0
fl0[0] = 1.0
let flat = diagonal(
    singles,
    fl0,
    fl => [0..=singles] k => p * fl[k] + (1.0 - p) * fl[max(k - 1, 0)],
    (fl, k) => fl[k],
)

@memo
fn win(k: 0..=singles, s: 0..=shines) -> F64 =
    if k == 0: level[s]
    elif s == 0: flat[k]
    else: p * win(k, s - 1) + (1.0 - p) * win(k - 1, s)
print(win(singles, shines))
