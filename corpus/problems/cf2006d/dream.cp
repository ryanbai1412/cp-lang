// Codeforces 2006 D: the best order alternates large and small values, so a window of length L needs,
// for every i <= sqrt(k), the values too big to sit beside anything above i matched by values <= i,
// up to floor(L / 2) of each, and one change fixes two mismatches. A value b > sqrt(k) matters only
// through k / b, so both sides are histograms over 1..=sqrt(k), kept per window by Mo's algorithm.
use std.mo.{mo}
@!multitest
let (n, q, k) = read<(2..=1e5, 1..=1e5, 1..=1e6)>()
domain P = 1..=n
domain Level = 1..=k.isqrt
enum Side: Small(Level) | Big(Level)
let b = read<[P]1..=k>()
let side: [P]Side = [P] i => if b[i] <= k.isqrt: Small(Level(b[i])) else: Big(Level(k / b[i]))
var small, big: [Level]Int = 0
small[1] += 1
let days = [d.l..=d.r for d in read<[q](l: P, r: P)>()]
fn step(i: P, d: Int):
    match side[i]:
        Small(l) => small[l] += d
        Big(l) => big[l] += d
fn answer(w: Range<P>) -> Int:
    var (best, need, have) = (0, 0, 0)
    for l in Level:
        need += big[l]
        have += small[l]
        best max= (need - have + Int(w.len + 1 > need + have) + 1) / 2
    best
print(mo(days, add: i => step(i, 1), remove: i => step(i, -1), answer: x => answer(days[x])))
