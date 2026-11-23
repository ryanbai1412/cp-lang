// A call whose operands are ordered passes a unit-typed argument (a stateless
// fn value), and printing a view of a struct with its own `show`.
use std.ntt.{convolve}
fn twice(v: Int) -> Int = v * 2
var x = List<Int>()
for i in 0..4: x.push(i + 1)
print(convolve(x.map(twice), x.map(twice).reversed(), cyclic: true))
type Z = Mod<998_244_353>
var a = List<Z>()
for i in 0..4: a.push(Z(i + 1) / Z(2))
print(a[1..3])
