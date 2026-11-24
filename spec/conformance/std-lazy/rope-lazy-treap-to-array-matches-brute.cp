// expect: ok, output "ok"
// rule: std.lazy.Rope
// rule: std.lazy.LazyTreap
use std.lazy.{LazyTreap, Rope}
domain I = 0..11
var x = 3
var a = [I] i => i.ord * 3
var b = [I] i => i.ord * 3
var c: List<Int> = [i * 3 for i in 0..11]
var add = Rope(a, merge: (t, v) => t + v)
var put = Rope(b)
var tr = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f)
for v in c: tr.push(v)
var bad = 0
for _ in 0..300:
    x = (x * 1103515245 + 12345) % 2147483648
    let p = x % 11
    x = (x * 1103515245 + 12345) % 2147483648
    let q = x % 11
    x = (x * 1103515245 + 12345) % 2147483648
    let v = x % 50
    let (lo, hi) = (min(p, q), max(p, q))
    if (x / 64) % 2 == 0:
        add.apply(I(lo)..=I(hi), v)
        put.apply(I(lo)..=I(hi), v)
        tr.apply(lo..hi + 1, v)
        for i in lo..=hi:
            a[I(i)] += v
            b[I(i)] = v
            c[i] += v
    else:
        add.reverse(I(lo)..=I(hi))
        put.reverse(I(lo)..=I(hi))
        tr.reverse(lo..hi + 1)
        for k in 0..(hi - lo + 1) / 2:
            let (i, j) = (I(lo + k), I(hi - k))
            (a[i], a[j]) = (a[j], a[i])
            (b[i], b[j]) = (b[j], b[i])
            (c[lo + k], c[hi - k]) = (c[hi - k], c[lo + k])
    if add.to_array() != a or put.to_array() != b or tr.to_list() != c: bad += 1
if "{add}" != " ".join(["{a[i]}" for i in I]) or "{tr}" != " ".join(["{v}" for v in c]): bad += 1
print(if bad == 0: "ok" else: "{bad} mismatches")
