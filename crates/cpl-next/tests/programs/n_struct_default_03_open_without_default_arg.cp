// expect: ok, output "0 0"
// rule: R08.2.5
// A type parameter nothing in the body fixes takes the struct's default, here read only by
// `new`'s body.
use std.default.Default
pub struct B<K, V: Default = Int>:
    n: Int
    init: V
    pub fn new(xs: List<K>) -> B<K, V> = B(0, V.default())
let b = B([1])
print(b.n, b.init)
