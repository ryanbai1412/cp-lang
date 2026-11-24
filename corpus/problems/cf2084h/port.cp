// Source: Codeforces 2084H (Turtle and Nediam 2), https://codeforces.com/contest/2084/problem/H
// limits: 2000 ms, 500 MB
// Problem: a binary string s of length n. An operation picks three consecutive s_i..s_i+2,
//   takes their median x and deletes the first s_j = x with j >= i. Count the distinct strings
//   reachable with at most n - 2 operations, mod 1e9+7. n, sum n <= 2e6; multitest.
// Solution: work on the run lengths `blocks` of s (one run: lengths 2..n, n - 1 strings). A DP
//   over runs alternates parity: for each height h the transition sums the dp of the newest
//   earlier run of the same parity that is at least h tall, and every earlier run grows by
//   one per round. Per parity a monotone `Staircase` stack stores tops minus the shared `lift`
//   and keeps the visible sum `hidden` up to date on push. The last run's length multiplies
//   the result. O(n) amortised.
// Bounds: n: 3..=2_000_000
@!multitest
use std.seq.runs

type M = Mod<1_000_000_007>
domain Parity = 0..2

// At height h the newest step at least h tall is visible. Every step grows by 1
// each round (`lift`), so tops are stored minus the lift and never rewritten.
struct Staircase:
    steps: List<(top: Int, v: M)>
    var lift: Int
    hidden: M

    fn push(mut self, height: Int, v: M):
        let top = height - lift
        while let cur = steps.last and cur.top <= top:
            steps.pop()
            if let b = steps.last: hidden -= M(b.top - cur.top) * b.v
        if let b = steps.last: hidden += M(b.top - top) * b.v
        steps.push((top, v))

    fn visible(self, floor: Int) -> M =
        hidden + M(floor + lift - 1) + (steps.last.map(s => M(s.top + lift) * s.v) ?? 0)

let n: 3..=2_000_000 = read()
let blocks = read<Str>().runs().map(r => r.len)
if blocks.len == 1:
    print(M(n - 1))
    return

var stairs = [Parity] _ => Staircase([], 0, 0)
var dp: M = 1
for k in 0..blocks.len - 1:
    let p = Parity(k % 2)
    stairs[p].lift += 1
    if k >= 1: stairs[p].push(blocks[k], dp)
    dp = stairs[p].visible(if p == Parity.first: blocks.first! else: 1)
print(dp * M(blocks.last!))
