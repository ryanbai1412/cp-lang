// A domain's `.bounds` is a range of cuts: kept, measured, mapped and
// reversed like any range, in concrete and generic code.
fn weight<D>(a: [D]Int) -> Int:
    let cuts = D.bounds
    cuts.map(b => b.ord).sum() + D.bounds.len

let n = 4
domain V = 0..n
let a: [V]Int = 0
let r = V.bounds
print(weight(a), r.len)
print(r.map(b => b.ord), V.bounds.rev.map(b => b.ord))
print([b.ord * 2 for b in V.bounds.rev])
print(V.bounds.first(b => b.ord >= 2)!.ord)

type Cut = V.bounds
fn digits(order: List<Cut>) -> Int:
    var t = 0
    for c in order:
        t = t * 10 + c.ord
    t
fn span(cuts: Range<Cut>) -> Int = cuts.len
print(digits(V.bounds), digits(V.bounds.rev), span(V.bounds))

// `.bounds` is never empty, so its first and last cuts need no `!`.
fn ends<D>(a: [D]Int) -> Int = D.bounds.last.ord * 10 + D.bounds.first.ord
var pre: [V.bounds]Int = 0
let top: Cut = V.bounds.last
pre[top] = 7
print(ends(a), pre[V.bounds.last], V.bounds.first.ord)
