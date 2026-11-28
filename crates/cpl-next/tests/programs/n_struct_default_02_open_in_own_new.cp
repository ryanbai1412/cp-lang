// expect: ok, output "0 0\n1 x"
// rule: R08.2.5
// A call of a struct's own `new` that leaves a type parameter open settles it to the
// parameter's default before a default argument naming it is read; an argument that says
// what it is wins.
use std.default.Default
pub struct B<K, V: Default = Int>:
    n: Int
    init: V
    pub fn new(xs: List<K>; init: V = V.default()) -> B<K, V> = B(xs.len - 1, init)
let b = B([1])
print(b.n, b.init)
let c = B([1, 2], init: "x")
print(c.n, c.init)
