// expect: ok
// rule: search-dp
// output: "4 8 3\n1.5000000000 2.0000000000 YES\n8 2.5000000000\n"
// One `bsearch`: the element type's `Search` picks what it finds, a cut on a domain or `Int`
// range and `?F64` on a `Range<F64>`, and a generic fn names it as `D.Found`.
use std.range.{Search}
domain V = 0..10
fn edge<D: Search>(r: Range<D>, ok: D -> Bool) -> D.Found = r.bsearch(ok)
let v = (V.first!..=V.last!).bsearch(x => x.ord >= 4).r!.ord
print(v, (0..100).bsearch(x => x * x >= 50).r, (0..10).bsearch(x => x <= 3, last: true).l)
let hi = (0.0..4.0).bsearch(x => x <= 1.5, last: true, steps: 60)!
print(hi, (0.0..8.0).bsearch(x => x >= 2.0)!, yes((0.0..1.0).bsearch(x => x > 2.0) == None))
print(edge(0..100, x => x * x >= 50).r, edge(0.0..10.0, x => x >= 2.5)!)
