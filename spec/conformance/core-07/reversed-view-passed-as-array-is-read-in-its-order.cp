// expect: ok, output "20 20 10 0\n20 0\n0 10"
// A reversed view passed where a `[P]T` is wanted becomes that array in the view's order, so
// the fn reads the reversal; a slice keeps its source's keys (R07.3.5, R07.5.11).
domain P = 0..3
let a: [P]Int = [P] i => 10 * i.ord
fn first(xs: [P]Int) -> Int = xs[P(0)]
fn ends(xs: [P]Int) -> (Int, Int) = (xs[P(0)], xs[P(2)])
print(first(a.rev), [x for x in a.rev])
let r = a.rev
let (lo, hi) = ends(r)
print(lo, hi)
print(first(a), a[P(1)])
