@!multitest
use std.mo.{mo}
let (n, q, k): (2..=100_000, 1..=100_000, 1..=1_000_000) = read()
domain P = 1..=n
let b: [P]1..=k = read()
let days: [q](l: P, r: P) = read()

let root = isqrt(k)
let level = [P] i => if b[i] <= root: (small: true, at: b[i]) else: (small: false, at: k / b[i])

let windows = [d.l..=d.r for d in days]
var small: [root + 1]Int = 0
var big: [root + 1]Int = 0
small[1] = 1
fn add_pos(pos: P):
    let (is_small, at) = level[pos]
    if is_small: small[at] += 1 else: big[at] += 1
fn rem_pos(pos: P):
    let (is_small, at) = level[pos]
    if is_small: small[at] -= 1 else: big[at] -= 1
fn answer_q(x: Int) -> Int:
    var (best, need, have) = (0, 0, 0)
    let len = windows[x].len
    for i in 1..=root:
        need += big[i]
        have += small[i]
        best = max(best, (need - have + Int(len + 1 > need + have) + 1) / 2)
    best
print(mo(windows, add: add_pos, remove: rem_pos, answer: answer_q))
