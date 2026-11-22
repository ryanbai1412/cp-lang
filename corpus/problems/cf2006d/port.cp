// Codeforces 2006 D: Iris and Adjacent Products.
// For any threshold `i <= sqrt(k)`, every large value (`b > k / i`) can only
// sit beside small values (`b <= i`), so `need` large values require at least
// `need - 1` small separators; seeding `cnt[Side.Small, Level(1)] = 1` adds
// that +1 virtual separator so `have = actual_small + 1` and the deficit is
// `need - have`. Each change turns one large element into a small one (closing
// the deficit by 2), except if `len + 1 > need + have` (i.e. at least one
// middle element exists outside both sets, or equivalently when the boundary
// parity forces an extra step), adding `+1` inside the ceiling:
//   changes >= ceil((need - have + Int(len + 1 > need + have)) / 2).
// Values `b > sqrt(k)` matter only through `k / b`, so both sides are
// histograms over `Level = 1..=isqrt(k)` maintained by Mo's algorithm.

use std.mo.{mo}

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
