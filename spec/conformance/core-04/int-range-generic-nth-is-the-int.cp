// expect: ok, output "1 2"
// rule: 04.4.30
// A generic `D` bound to `Int` by `Range<Int>` arguments is the integer ranges' anonymous
// domain: its positions are the `Int`s, so `D.nth(k)` is `k`.
fn ends<D>(r: Range<D>) -> (D, D):
    let (l, h) = (r.l, r.r)
    (D.nth(l), D.nth(h - 1))
let (a, b) = ends(1..3)
print(a, b)
