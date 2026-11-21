@!multitest
use std.mo.{mo_order, mo_steps}
let (n, q, k): (2..=100_000, 1..=100_000, 1..=1_000_000) = read()
domain P = 1..=n
let b: [P]1..=k = read()
let days: [q](l: P, r: P) = read()

var root = 1
while (root + 1) * (root + 1) <= k: root += 1
let level = [P] i => if b[i] <= root: (small: true, at: b[i]) else: (small: false, at: k / b[i])

let windows = [d.l..=d.r for d in days]
var small: [root + 1]Int = 0
var big: [root + 1]Int = 0
small[1] = 1
var cur = P.first..P.first
var ans: [q]Int = 0
for x in mo_order(windows):
    for (pos, entering) in mo_steps(cur, windows[x]):
        let (is_small, at) = level[pos]
        let d = if entering: 1 else: -1
        if is_small: small[at] += d else: big[at] += d
    cur = windows[x]
    var (need, have) = (0, 0)
    for i in 1..=root:
        need += big[i]
        have += small[i]
        ans[x] = max(ans[x], (need - have + Int(cur.len + 1 > need + have) + 1) / 2)
print(ans)
