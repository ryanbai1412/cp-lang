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
