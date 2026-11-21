// A record of domains is a product domain, whatever its fields are named:
// `[P]T` over `(a: A, b: B)` is laid out and indexed like `[(A, B)]T`,
// written directly, through an alias, or as a generic.
domain A = 0..3
domain B = 0..2
type P = (a: A, b: B)
type Q = (A, B)
var named: [P]?Int = none
var plain: [Q]Int = 0
var k = 0
for a in A:
    for b in B:
        let p: P = (a: a, b: b)
        named[p] = k
        k += 1
        plain[(a, b)] += named[p]!
fn twice<S>(s: S, x: Int) -> Int:
    var seen: [S]?Int = none
    seen[s] = x
    seen[s]! * 2
let p: P = (a: A(2), b: B(1))
print(named[p]!, plain[(A(2), B(1))], twice(p, 4), twice((A(1), B(0)), 5))
print(named[(a: A(0), b: B(1))] ?? -1)
