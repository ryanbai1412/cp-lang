// expect: E0608
// rule: 05.9.1
// A lent generator over `a` is live for the walk, so the body may not write `a` (spec/core/05-ownership.md exercise 33).
domain V = 0..8
var a: [V]Int = 0
let l = V(2)
fn evens(xs: [V]Int) -> lends Seq<Int> = xs.filter(x => x % 2 == 0)
for x in evens(a): a[l] += x // <- here
print(a[l])
