// Count binary strings with x zeros and y ones, > a, having LNDS >= k.
// Walk a; at each '0', flipping it to '1' produces strings strictly > a.
// Before a's prefix reaches LNDS == k, accumulate diagonal difference updates
// where LNDS first hits k, sweep diagonals, and close each suffix with choose.
// After a's prefix reaches LNDS == k, reset and count each suffix directly.
@!multitest
type Z = Mod<998_244_353>

let (x, y, k): (1..=5000, 1..=5000, 1..=9999) = read()
let a: Str = read()
let comb = Z.comb(x + y)

struct Prefix(z: Int, o: Int, lnds: Int):
    fn push0() -> Prefix = Prefix(z + 1, o, max(lnds, z + 1))
    fn push1() -> Prefix = Prefix(z, o + 1, lnds + 1)

domain Cnt = 0..=k
var pref: [Cnt, Cnt]Z = 0
var ans: Z = 0

fn add_before_k(s: Prefix):
    if s.lnds == k:
        pref[s.z, s.o] += 1
        pref[s.z + 1, s.o - 1] -= 1
        return
    let (e0, e1) = (k - s.z, k - s.lnds)
    for d in e1..e0 + e1:
        let top = comb.choose(d, e1) - comb.choose(d - 1, e1) - comb.choose(d - 1, e0)
        let main = comb.choose(d, e0) - 2 * comb.choose(d - 1, e0)
        let (px, py) = (s.z + d - e1, s.o + e1)
        pref[px, py] += top
        pref[px + 1, py - 1] += main - top

var cur = Prefix(0, 0, 0)
var base: ?(Int, Int) = none

for ch in a:
    if let (ox, oy) = base:
        if ch == '0':
            let s = cur.push1()
            let (rx, ry) = (x - ox - s.z, y - oy - s.o)
            let (e0, e1) = (k - s.z, k - s.lnds)
            ans += comb.choose(rx + ry, if rx >= e0 or ry >= e1: rx else: e0)
        cur = if ch == '0': cur.push0() else: cur.push1()
    else:
        if ch == '0': add_before_k(cur.push1())
        cur = if ch == '0': cur.push0() else: cur.push1()
        if cur.lnds >= k:
            base = (cur.z, cur.o)
            cur = Prefix(0, 0, 0)

for i in 0..k, j in 1..=k:
    pref[i + 1, j - 1] += pref[i, j]

ans += sum(
    pref[px, py] * comb.choose(x + y - px - py, max(x - px, y - py, k))
    for px in Cnt, py in Cnt if pref[px, py] != 0
)
print(ans)
