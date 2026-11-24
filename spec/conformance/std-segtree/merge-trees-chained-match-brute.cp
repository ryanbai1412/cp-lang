// expect: ok, output "ok"
// rule: std.segtree.MergeTrees.reserve
// A reserved `MergeTrees` pool through chains of merges, scales and sets into merged trees, whose
// pending tags sit at every depth: get, prefix and inner sums and totals agree with plain arrays.
use std.segtree.{MergeTree, MergeTrees}

type Z = Mod<998244353>
domain D = 0..11

fn conv(a: [D]Z, b: [D]Z, pb: Z) -> [D]Z:
    [D] x => a[x] * (pb + sum(b[y] for y in D if y <= x)) + b[x] * sum(a[y] for y in D if y < x)

var ok = true
for round in 0..40:
    var p = MergeTrees<D, Z>()
    p.reserve(sets: rand(0, 30))
    var t = MergeTree<D>()
    var a: [D]Z = [D] _ => Z(0)
    for step in 0..8:
        var u = MergeTree<D>()
        var b: [D]Z = [D] _ => Z(0)
        for _ in 0..rand(0, 4):
            let x = D(rand(0, 10))
            let v = Z(rand(1, 1000))
            p.set(mut u, x, v)
            b[x] = v
        let k = Z(rand(1, 9))
        p.scale(u, k)
        for x in D: b[x] = b[x] * k
        let pb = Z(rand(0, 5))
        t = p.merge(t, u, (va, vb, qa, qb) => va * (qb + vb) + vb * qa, pre_b: pb)
        a = conv(a, b, pb)
        if rand(0, 1) == 0:
            let x = D(rand(0, 10))
            let v = Z(rand(0, 1000))
            p.set(mut t, x, v)
            a[x] = v
        for x in D:
            if p.get(t, x) != a[x]: ok = false
        for l in 0..=11:
            for r in l..=11:
                if p.sum(t, D.cuts[l]..D.cuts[r]) != sum(
                    a[x] for x in D if l <= x.ord < r
                ): ok = false
        if p.total(t) != sum(a[x] for x in D): ok = false
print(if ok: "ok" else: "mismatch")
