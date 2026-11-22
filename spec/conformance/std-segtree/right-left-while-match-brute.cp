// expect: ok, output "0 3040"
use std.segtree.{SegTree}
use std.rand.{Rng}
var rng = Rng(7)
var bad = 0
var checks = 0
for n in [1, 2, 3, 5, 8, 13]:
    domain P = 0..n
    for _ in 0..40:
        var a: [P]Int = 0
        for i in P: a[i] = rng.int(0, 5)
        let s = SegTree<P, Int>.of(+, i => a[i])
        let lim = rng.int(0, 12)
        for f in 0..=n:
            var d = f
            var sum = 0
            while d < n and sum + a[P.nth(d)] <= lim:
                sum += a[P.nth(d)]
                d += 1
            checks += 2
            if s.right_while(P.cuts.nth(f), x => x <= lim).ord != d: bad += 1
            var e = f
            var sm = 0
            while e > 0 and sm + a[P.nth(e - 1)] <= lim:
                sm += a[P.nth(e - 1)]
                e -= 1
            if s.left_while(P.cuts.nth(f), x => x <= lim).ord != e: bad += 1
print(bad, checks)
