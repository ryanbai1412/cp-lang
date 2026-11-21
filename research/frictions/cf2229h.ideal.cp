// Ideal CP Lang program for cf2229h (H. Wowee Binary String).
// Not gated: upstream ecnerwala/contest-programming CF/2229/H/H.cpp is an
// unfinished draft (duplicate `S[i] == '?'` branch, empty transition block,
// and no output statement).

@!multitest
let n: 1..=3000 = read()
let s: String = read()
type M = Mod<998_244_353>

// A binary string t is obtainable iff it can be greedily matched as a subsequence
// of some completion of s where every skipped gap (and trailing suffix) has an
// even number of '1's (or contains a '?' which can absorb any parity).
// State (i, j) tracks the earliest position reachable with even parity (i) and
// with odd parity (j) of prefix '1's; position n + 1 denotes "unreachable".
var nxt_q: [0..=n]Int = n
var nxt_1: [0..=n]Int = n
var nxt_0: [0..=n, 0..2]Int = n
var suff_1: [0..=n]Int = 0

for i in (0..n).rev():
    suff_1[i] = suff_1[i + 1] ^ (if s[i] == '1': 1 else: 0)
    match s[i]:
        '?' =>
            nxt_q[i] = i
            nxt_1[i] = i
            nxt_0[i, 0] = i
            nxt_0[i, 1] = nxt_0[i + 1, 1]
        '1' =>
            nxt_q[i] = nxt_q[i + 1]
            nxt_1[i] = i
            nxt_0[i, 0] = nxt_0[i + 1, 1]
            nxt_0[i, 1] = nxt_0[i + 1, 0]
        '0' =>
            nxt_q[i] = nxt_q[i + 1]
            nxt_1[i] = nxt_1[i + 1]
            nxt_0[i, 0] = i
            nxt_0[i, 1] = nxt_0[i + 1, 1]

fn step_0(pos: Int) -> (even: Int, odd: Int):
    if pos > n: return (even: n + 1, odd: n + 1)
    let e = min(nxt_0[pos, 0], nxt_q[pos])
    let o = min(nxt_0[pos, 1], nxt_q[pos])
    return (
        even: if e < n: e + 1 else: n + 1,
        odd: if o < n: o + 1 else: n + 1,
    )

fn step_1(pos: Int) -> (even: Int, odd: Int):
    if pos > n: return (even: n + 1, odd: n + 1)
    let p = nxt_1[pos]
    let q = nxt_q[pos]
    return (
        even: if p < n: p + 1 else: n + 1,
        odd: if q < n: q + 1 else: n + 1,
    )

var dp: [0..=n + 1, 0..=n + 1]M = M(0)
dp[0, n + 1] = M(1)
var ans = M(0)

for i in 0..=n + 1:
    for j in 0..=n + 1:
        let cur = dp[i, j]
        if cur == M(0): continue
        let can_end =
            (i <= n and (suff_1[i] == 0 or nxt_q[i] < n)) or
            (j <= n and (suff_1[j] == 1 or nxt_q[j] < n))
        if can_end: ans += cur

        let a0 = step_0(i)
        let b0 = step_0(j)
        dp[min(a0.even, b0.odd), min(a0.odd, b0.even)] += cur

        let a1 = step_1(i)
        let b1 = step_1(j)
        dp[min(a1.even, b1.odd), min(a1.odd, b1.even)] += cur

print(ans)
