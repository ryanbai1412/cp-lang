// Codeforces 2006 D: the best order alternates large and small values, so a window of length L needs,
// for every i <= sqrt(k), the values too big to sit beside anything above i matched by values <= i,
// up to floor(L / 2) of each, and one change fixes two mismatches. A value b > sqrt(k) matters only
// through k / b, so both sides are histograms over 1..=sqrt(k), kept per window by Mo's algorithm.
@!multitest
use std.mo.{mo}
let (n, q, k): (2..=100_000, 1..=100_000, 1..=1_000_000) = read()
domain P = 1..=n
let b: [P]1..=k = read()
let days: [q](l: P, r: P) = read()

let root = isqrt(k)
domain Level = 1..=root
let level = [P] i => if b[i] <= root: (small: true, at: Level(b[i])) else: (
    small: false,
    at: Level(k / b[i]),
)

let windows = [d.l..=d.r for d in days]
var small: [Level]Int = 0
var big: [Level]Int = 0
small[Level(1)] = 1
fn add_pos(pos: P):
    let (is_small, at) = level[pos]
    if is_small: small[at] += 1 else: big[at] += 1
fn rem_pos(pos: P):
    let (is_small, at) = level[pos]
    if is_small: small[at] -= 1 else: big[at] -= 1
fn answer_q(x: Int) -> Int:
    var (best, need, have) = (0, 0, 0)
    let len = windows[x].len
    for i in Level:
        need += big[i]
        have += small[i]
        best max= (need - have + Int(len + 1 > need + have) + 1) / 2
    best
print(mo(windows, add: add_pos, remove: rem_pos, answer: answer_q))
