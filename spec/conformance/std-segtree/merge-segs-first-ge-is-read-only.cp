// expect: ok, output "YES"
// MergeSegs.first_ge and any_in answer from a merged set without changing it or its inputs,
// checked against sets kept as [D]Bool after every merge.
use std.segtree.{MergeSeg, MergeSegs}

domain D = 0..40
var ok = true
for trial in 0..30:
    var p = MergeSegs<D>()
    var segs: List<MergeSeg<D>> = []
    var want: List<[D]Bool> = []
    for _ in 0..8:
        let x = D(rand(0, 39))
        segs.push(p.singleton(x))
        want.push([D] d => d == x)
    for _ in 0..12:
        let i = rand(0, segs.len - 1)
        let j = rand(0, segs.len - 1)
        segs.push(p.merge(segs[i], segs[j]))
        let (a, b) = (want[i], want[j])
        want.push([D] d => a[d] or b[d])
    for _ in 0..3:
        for k in 0..segs.len:
            let x = D(rand(0, 39))
            let lo = rand(0, 39)
            let hi = rand(lo, 39)
            let first = [d for d in D if want[k][d] and d >= x].first
            if p.first_ge(segs[k], x) != first: ok = false
            let r = D(lo)..D(hi)
            let any = any(want[k][d] for d in D if lo <= d.ord < hi)
            if p.any_in(segs[k], r) != any: ok = false
print(yes(ok))
