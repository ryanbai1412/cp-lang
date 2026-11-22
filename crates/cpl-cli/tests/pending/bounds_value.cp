// A domain's `.cuts` is a range of cuts: kept, measured, mapped and
// reversed like any range, in concrete and generic code.
fn weight<D>(a: [D]Int) -> Int:
    let cuts = D.cuts
    cuts.map(b => b.ord).sum() + D.cuts.len

let n = 4
domain V = 0..n
let a: [V]Int = 0
let r = V.cuts
print(weight(a), r.len)
print(r.map(b => b.ord), V.cuts.rev.map(b => b.ord))
print([b.ord * 2 for b in V.cuts.rev])
print(V.cuts.first(b => b.ord >= 2)!.ord)

type C = V.cuts
fn digits(order: List<C>) -> Int:
    var t = 0
    for c in order:
        t = t * 10 + c.ord
    t
fn span(cuts: Range<C>) -> Int = cuts.len
print(digits(V.cuts), digits(V.cuts.rev), span(V.cuts))

// `.cuts` is never empty, so its first and last cuts need no `!`.
fn ends<D>(a: [D]Int) -> Int = D.cuts.last.ord * 10 + D.cuts.first.ord
var pre: [V.cuts]Int = 0
let top: C = V.cuts.last
pre[top] = 7
print(ends(a), pre[V.cuts.last], V.cuts.first.ord)
