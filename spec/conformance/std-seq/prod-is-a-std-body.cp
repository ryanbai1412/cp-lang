// expect: ok, output "24 24 120\n1\n70 14\n401576539"
// rule: std.seq.prod
// `prod` is the std body in `std.seq`, not a compiler builtin: the method,
// the free call, a generator, an empty list, a ranged array and `Mod<P>`.
use std.mod.{Mod}
type Z = Mod<998244353>
print([2, 3, 4].prod(), prod([2, 3, 4]), prod(x for x in 1..=5))
let e: List<Int> = []
print(e.prod())
let a: [1..=3]Int = [2, 5, 7]
print(a.prod(), prod(a[i] for i in 1..=3 if i != 2))
print(prod(Z(i) for i in 1..=20))
