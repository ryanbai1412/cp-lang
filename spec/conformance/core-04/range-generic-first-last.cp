// expect: ok, output "1 3 2 0"
// rule: 04.8.4
// `r.first` / `r.last` on a `Range<D>` over a generic domain `D` are `?D`s, so their ranks type.
domain V = 0..5
fn ends<D: Domain>(r: Range<D>) -> (Int, Int) = (r.first!.ord, r.last!.ord + 1)
fn width<D: Domain>(r: Range<D>) -> Int:
    if r.is_empty: return 0
    let (lo, hi) = ends(r)
    hi - lo
let (lo, hi) = ends(V(1)..V(3))
print(lo, hi, width(V(1)..V(3)), width(V(2)..V(2)))
