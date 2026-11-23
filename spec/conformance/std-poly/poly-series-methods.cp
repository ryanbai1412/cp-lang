// expect: ok, output "1 1 1 1 | 0 2 998244352 | 1 2 2 | 1 3 3 1 | 1 1 0 | 1 2 1 | 3 5 7 | 3 2 | 1 0 1"
// `Poly<T>` methods work over any field `T` and agree with the free functions on `List<Mod<P>>`.
use std.poly.{Poly}
type Z = Mod<998244353>
fn show(p: Poly<Z>) -> Str = " ".join([Str(p.at(i).v) for i in 0..p.len])
let a = Poly<Z>([1, -1])
let q = Poly<Z>([1, 2, 1])
print(show(a.inv(4)), "|", show(q.log(3)), "|", show(Poly<Z>([0, 2]).exp(3)), "|", show(Poly<Z>([1, 1]).pow(3, 4)), "|", show(q.sqrt(3)!), "|", show(Poly<Z>([0, 0, 1]).shift(Z(1))), "|", show(Poly<Z>([1, 2, 3, 4]).transposed_mul(Poly<Z>([1, 1]))), "|", show(Poly<Z>([3, 5, 2]) / Poly<Z>([1, 1])), "|", show(Poly<Z>([1, 0, 1]) * Poly<Z>.one))
