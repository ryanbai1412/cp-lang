// expect: ok, output "0 YES"
// rule: std.lazy.LazyTreap.reverse
// A reversed run re-ranks its pending `a + d * k` tags through `mirror`: 3000 random pushes,
// reverses, rank-dependent applies and reads agree with a plain list.
use std.lazy.{LazyTreap}
use std.rand.{Rng}

var t = LazyTreap<Int, (Int, Int)>(
    apply: (v, k, f) => v + f.0 + f.1 * k,
    compose: (g, f) => (g.0 + f.0, g.1 + f.1),
    split: (f, k) => (f, (f.0 + f.1 * (k + 1), f.1)),
    mirror: (f, n) => (f.0 + f.1 * (n - 1), -f.1),
)
var m: List<Int> = []
var rng = Rng(7)
var bad = 0
for _ in 0..3000:
    let op = rng.int(0, 4 - 1)
    if op == 0 or m.len < 2:
        let x = rng.int(0, 100 - 1)
        t.push(x)
        m.push(x)
    else:
        let lo = rng.int(0, m.len - 1)
        let hi = lo + 1 + rng.int(0, m.len - lo - 1)
        if op == 1:
            t.reverse(lo..hi)
            var i = lo
            var j = hi - 1
            while i < j:
                let s = m[i]
                m[i] = m[j]
                m[j] = s
                i += 1
                j -= 1
        elif op == 2:
            let a = rng.int(0, 10 - 1)
            let d = rng.int(0, 5 - 1)
            t.apply(lo..hi, (a, d))
            for k in lo..hi: m[k] += a + d * (k - lo)
        else:
            if t.get(lo) != m[lo]: bad += 1
print(bad, yes(t.to_list() == m))
