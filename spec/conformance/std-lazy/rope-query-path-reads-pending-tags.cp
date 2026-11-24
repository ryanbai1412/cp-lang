// expect: ok, output "400 400\n3"
// `query_path` sees every piece of a position's value (its stored value, then the tags pending
// above it) without pushing tags down, and a piece `f` changes stays changed.
use std.lazy.{Rope}
domain V = 0..30
var seed = 11
fn rnd(mut seed: Int, k: Int) -> Int:
    seed = (seed * 1103515245 + 12345) % 2147483648
    (seed / 65536) % k
var a: [V]Int = 0
var r = Rope<V, Int>(fill: 0, merge: (t, x) => t + x)
var (ok, ok2) = (0, 0)
for _ in 0..400:
    let lo = rnd(mut seed, 30)
    let hi = lo + rnd(mut seed, 30 - lo)
    if rnd(mut seed, 2) == 0:
        let t = rnd(mut seed, 9) + 1
        r.apply(V(lo)..=V(hi), t)
        for i in lo..=hi: a[V(i)] += t
    else:
        r.reverse(V(lo)..=V(hi))
        var (i, j) = (lo, hi)
        while i < j:
            let tmp = a[V(i)]
            a[V(i)] = a[V(j)]
            a[V(j)] = tmp
            i += 1
            j -= 1
    let p = V(rnd(mut seed, 30))
    var sum = 0
    let none: ?Int = r.query_path(p, (mut x) =>
        sum += x
        None)
    if none == None and sum == a[p]: ok += 1
    if r.get(p) == a[p]: ok2 += 1
print(ok, ok2)
var s = Rope([1, 2, 3], merge: (t, x) => t + x)
s.apply(0..3, 10)
let first = s.query_path(2, (mut x) =>
    x = 0
    Some(1))
let after = s.query_path(2, (mut x) => if x == 0: Some(3) else: None)
print((first ?? 0) * (after ?? 0))
