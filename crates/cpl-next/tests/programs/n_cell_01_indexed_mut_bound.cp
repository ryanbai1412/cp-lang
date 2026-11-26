// expect: ok, output "3 c 4"
// rule: R12.4.6
use std.seq.{Indexed}
fn first<S: IndexedMut<Int, T>, T>(s: S) -> T = s[0]
domain V = 0..2
var a: [V]Int = 4
print(first([3, 1]), first("cd"), a[V(1)])
