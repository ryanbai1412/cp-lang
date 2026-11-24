// expect: ok, output "YES"
// `reverse` mixed with a rank-dependent tag (add `a + d*k` at rank `k`), inserts, erases and
// reads agrees with a plain list.
use std.lazy.{LazyTreap}
var ok = true
for trial in 0..60:
    var t = LazyTreap<Int, (a: Int, d: Int)>(
        apply: (v, k, f) => v + f.a + f.d * k,
        compose: (g, f) => (a: g.a + f.a, d: g.d + f.d),
        split: (f, k) => (f, (a: f.a + f.d * (k + 1), d: f.d)),
        mirror: (f, n) => (a: f.a + f.d * (n - 1), d: -f.d),
    )
    var b: List<Int> = []
    for step in 0..40:
        let op = rand(0, 6)
        if op <= 1 or b.len == 0:
            let pos = rand(0, b.len)
            let x = rand(-9, 9)
            t.insert_at(pos, x)
            b.insert(pos, x)
        elif op == 2:
            let lo = rand(0, b.len)
            let hi = rand(lo, b.len)
            t.reverse(lo..hi)
            var i = lo
            var j = hi - 1
            while i < j:
                let x = b[i]
                b[i] = b[j]
                b[j] = x
                i += 1
                j -= 1
        elif op == 3:
            let lo = rand(0, b.len)
            let hi = rand(lo, b.len)
            let f = (a: rand(-5, 5), d: rand(-3, 3))
            t.apply(lo..hi, f)
            for k in lo..hi: b[k] += f.a + f.d * (k - lo)
        elif op == 4:
            let pos = rand(0, b.len - 1)
            if t.erase_at(pos) != b[pos]: ok = false
            b.remove(pos)
        elif op == 5:
            let k = rand(0, b.len)
            if t.take(k) != b.take(k): ok = false
        else:
            let pos = rand(0, b.len - 1)
            if t.get(pos) != b[pos]: ok = false
        if t.to_list() != b: ok = false
print(if ok: "YES" else: "NO")
