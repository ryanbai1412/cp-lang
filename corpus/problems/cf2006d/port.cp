// Source: Codeforces 2006 D, Iris and Adjacent Products,
//   https://codeforces.com/contest/2006/problem/D
// Problem: multitest: an array b of n values in 1..k (n, q <= 1e5 summed, k <= 1e6). For each
//   of q subarrays b[l..r] print the fewest elements to change (to any value in 1..k) so that
//   some rearrangement has every adjacent product <= k.
// Solution: For every threshold i <= sqrt(k), a value > k / (i + 1) can only sit beside values
//   <= i, so the large values need about as many small separators; each change turns a large
//   value small and closes the gap by 2, and the answer is the max deficit over i. Values
//   > sqrt(k) matter only through k / b, so a Small and a Big histogram over 1..=isqrt(k) are
//   kept by Mo's algorithm and scanned per query: O(n sqrt(q) + q sqrt(k)).
// Bounds: n: 2..=100_000; q: 1..=100_000; k: 1..=1_000_000; b: 1..=k; days.l, days.r: 1..=n


enum Side: Small | Big

@!multitest
let (n, q, k): (2..=100_000, 1..=100_000, 1..=1_000_000) = read()
domain P = 1..=n
let b: [P]1..=k = read()
let days: [q](l: P, r: P) = read()

let root = isqrt(k)
domain Level = 1..=root
let level = [P] i => if b[i] <= root: (side: Side.Small, at: Level(b[i])) else: (
    side: Side.Big,
    at: Level(k / b[i]),
)

let windows = [d.l..=d.r for d in days]
var cnt: [Side, Level]Int = 0
cnt[Side.Small, Level(1)] = 1
fn adjust(pos: P, delta: Int):
    let (side, at) = level[pos]
    cnt[side, at] += delta
fn answer_q(x: Int) -> Int:
    var (best, need, have) = (0, 0, 0)
    let len = windows[x].len
    for i in Level:
        need += cnt[Side.Big, i]
        have += cnt[Side.Small, i]
        best max= (need - have + Int(len + 1 > need + have) + 1) / 2
    best
print(mo(windows, add: p => adjust(p, 1), remove: p => adjust(p, -1), answer: answer_q))
