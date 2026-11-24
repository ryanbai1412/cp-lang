// expect: ok, output "ok"
// rule: std.segtree.MergeTrees.merge
// `MergeTrees` set/sum/scale/merge agree with plain arrays, including the max-convolution merge
// `new[x] = a[x] * B(<= x) + b[x] * A(< x)` that tree DP (noi20p2) runs on each child.
use std.segtree.{MergeTree, MergeTrees}

type Z = Mod<998244353>
domain D = 0..13

fn conv(a: [D]Z, b: [D]Z) -> [D]Z:
    [D] x => a[x] * sum(b[y] for y in D if y <= x) + b[x] * sum(a[y] for y in D if y < x)

var ok = true
for round in 0..60:
    var p = MergeTrees<D, Z>()
    var ta = MergeTree<D>()
    var tb = MergeTree<D>()
    var a: [D]Z = [D] _ => Z(0)
    var b: [D]Z = [D] _ => Z(0)
    for _ in 0..rand(0, 6):
        let x = D(rand(0, 12))
        let v = Z(rand(1, 1000))
        p.set(mut ta, x, v)
        a[x] = v
    for _ in 0..rand(0, 6):
        let x = D(rand(0, 12))
        let v = Z(rand(1, 1000))
        p.set(mut tb, x, v)
        b[x] = v
    let k = Z(rand(2, 9))
    p.scale(ta, k)
    for x in D: a[x] = a[x] * k
    let m = p.merge(ta, tb, (va, vb, pa, pb) => va * (pb + vb) + vb * pa)
    let want = conv(a, b)
    for x in D:
        if p.get(m, x) != want[x]: ok = false
    for l in 0..=13:
        for r in l..=13:
            if p.sum(m, D.cuts[l]..D.cuts[r]) != sum(want[x] for x in D if l <= x.ord < r): ok = false
    if p.total(m) != sum(want[x] for x in D): ok = false
print(if ok: "ok" else: "mismatch")
