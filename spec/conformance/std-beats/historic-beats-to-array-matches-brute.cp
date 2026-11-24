// expect: ok, output "ok"
// rule: std.beats.HistoricBeatsSegTree
use std.beats.{HistoricBeatsSegTree}
domain I = 0..13
var x = 7
var a = [I] i => (i.ord * 5) % 11 - 5
var t = HistoricBeatsSegTree<I>(a)
var bad = 0
for step in 0..300:
    x = (x * 1103515245 + 12345) % 2147483648
    let p = x % 13
    x = (x * 1103515245 + 12345) % 2147483648
    let q = x % 13
    x = (x * 1103515245 + 12345) % 2147483648
    let v = x % 21 - 10
    let (lo, hi) = (min(p, q), max(p, q))
    let kind = (x / 64) % 3
    if kind == 0: t.chmin(I(lo)..=I(hi), v)
    elif kind == 1: t.chmax(I(lo)..=I(hi), v)
    else: t.add(I(lo)..=I(hi), v)
    for i in lo..=hi:
        let k = I(i)
        a[k] = if kind == 0: min(a[k], v) elif kind == 1: max(a[k], v) else: a[k] + v
    if t.to_array() != a: bad += 1
    if step % 7 == 0 and t.sum(I(lo)..=I(hi)) != (lo..=hi).map(i => a[I(i)]).sum(): bad += 1
let want = " ".join(["{a[i]}" for i in I])
if "{t}" != want: bad += 1
print(if bad == 0: "ok" else: "{bad} mismatches")
