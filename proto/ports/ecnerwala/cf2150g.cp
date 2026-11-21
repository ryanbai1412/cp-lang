@!multitest
use std.mod.{Comb, Mod}

type Z = Mod<998_244_353>

struct State(cx: Int, cy: Int, lnds: Int)

let (x, y, k): (1..=5000, 1..=5000, 1..=9999) = read()
let n = x + y
let a: Str = read()
let comb = Comb<998_244_353>(n)

domain Cnt = 0..=k
var pref: [Cnt, Cnt]Z = Z.zero
var ans = Z.zero

fn add_first_half(s: State):
    if s.lnds == k:
        pref[Cnt(s.cx), Cnt(s.cy)] += Z.one
        pref[Cnt(s.cx + 1), Cnt(s.cy - 1)] -= Z.one
        return
    let e0 = k - s.cx
    let e1 = k - s.lnds
    for d in e1..e0 + e1:
        let ways_top = comb.choose(d, e1) - comb.choose(d - 1, e1) - comb.choose(d - 1, e0)
        let ways_main = comb.choose(d, e0) - Z(2) * comb.choose(d - 1, e0)
        let px = s.cx + d - e1
        let py = s.cy + e1
        pref[Cnt(px), Cnt(py)] += ways_top
        pref[Cnt(px + 1), Cnt(py - 1)] += ways_main - ways_top

var cur = State(0, 0, 0)
var pos = 0
while pos < a.len:
    let ch = a[pos]
    pos += 1
    if ch == '0':
        add_first_half(State(cur.cx, cur.cy + 1, cur.lnds + 1))
        cur = State(cur.cx + 1, cur.cy, max(cur.lnds, cur.cx + 1))
    else:
        cur = State(cur.cx, cur.cy + 1, cur.lnds + 1)
    if cur.lnds >= k: break

if cur.lnds >= k:
    let ox = cur.cx
    let oy = cur.cy
    cur = State(0, 0, 0)
    while pos < a.len:
        let ch = a[pos]
        pos += 1
        if ch == '0':
            let cnd = State(cur.cx, cur.cy + 1, cur.lnds + 1)
            let rx = x - ox - cnd.cx
            let ry = y - oy - cnd.cy
            let e0 = k - cnd.cx
            let e1 = k - cnd.lnds
            ans += if rx >= e0 or ry >= e1: comb.choose(rx + ry, rx) else: comb.choose(rx + ry, e0)
            cur = State(cur.cx + 1, cur.cy, max(cur.lnds, cur.cx + 1))
        else:
            cur = State(cur.cx, cur.cy + 1, cur.lnds + 1)

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
