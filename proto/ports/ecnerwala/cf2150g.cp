// Count binary strings with x zeros and y ones, > a, having LNDS >= k.
// Walk a; flipping a '0' to '1' produces strings strictly > a.
// Before a's prefix reaches LNDS == k, accumulate diagonal difference updates
// where LNDS first hits k, sweep diagonals, and close each suffix with choose.
// After a's prefix reaches LNDS == k, reset and count each suffix directly.
@!multitest
use std.mod.{Comb, Mod}

const MOD = 998_244_353
type Z = Mod<MOD>

struct Prefix(z: Int, o: Int, lnds: Int)

fn push0(s: Prefix) -> Prefix = Prefix(s.z + 1, s.o, max(s.lnds, s.z + 1))
fn push1(s: Prefix) -> Prefix = Prefix(s.z, s.o + 1, s.lnds + 1)

let (x, y, k): (1..=5000, 1..=5000, 1..=9999) = read()
let n = x + y
let a: Str = read()
let comb = Comb<Z>(n)

domain Cnt = 0..=k
var pref: [Cnt, Cnt]Z = Z.zero
var ans = Z.zero

fn add_before_k(s: Prefix):
    if s.lnds == k:
        pref[Cnt(s.z), Cnt(s.o)] += Z.one
        pref[Cnt(s.z + 1), Cnt(s.o - 1)] -= Z.one
        return
    let (e0, e1) = (k - s.z, k - s.lnds)
    for d in e1..e0 + e1:
        let top = comb.choose(d, e1) - comb.choose(d - 1, e1) - comb.choose(d - 1, e0)
        let main = comb.choose(d, e0) - Z(2) * comb.choose(d - 1, e0)
        let (px, py) = (s.z + d - e1, s.o + e1)
        pref[Cnt(px), Cnt(py)] += top
        pref[Cnt(px + 1), Cnt(py - 1)] += main - top

var cur = Prefix(0, 0, 0)
var base: ?(Int, Int) = none

for ch in a:
    if let (ox, oy) = base:
        if ch == '0':
            let s = push1(cur)
            let (rx, ry) = (x - ox - s.z, y - oy - s.o)
            let (e0, e1) = (k - s.z, k - s.lnds)
            ans += comb.choose(rx + ry, if rx >= e0 or ry >= e1: rx else: e0)
        cur = if ch == '0': push0(cur) else: push1(cur)
    else:
        if ch == '0': add_before_k(push1(cur))
        cur = if ch == '0': push0(cur) else: push1(cur)
        if cur.lnds >= k:
            base = (cur.z, cur.o)
            cur = Prefix(0, 0, 0)

for i in 0..k:
    for j in 1..=k:
        pref[Cnt(i + 1), Cnt(j - 1)] += pref[Cnt(i), Cnt(j)]

for px in Cnt:
    for py in Cnt:
        let w = pref[px, py]
        if w != Z.zero:
            ans += w * comb.choose(
                n - px.to_idx - py.to_idx,
                max(x - px.to_idx, max(y - py.to_idx, k)),
            )

print(ans)
