// expect: ok, output "2 YES"
// rule: 04.8.4
// `r.start` / `r.end` on a `Range<D>` over a generic domain `D` are `D`s, as for a declared domain.
domain V = 0..5
fn width<D: Domain>(r: Range<D>) -> Int = r.end.idx - r.start.idx
fn ordered<D: Domain>(r: Range<D>) -> Bool = r.start <= r.end
print(width(V(1)..V(3)), yes(ordered(V(1)..V(3))))
