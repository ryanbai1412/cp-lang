// expect: ok, output "YES\n1 12 NO\n6 17 8\n4"
use std.lazy.{LazyTreap}
var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f)
print(t.is_empty)
t.push(1)
t.push(2)
t.apply(1..2, 10)
print(t.to_list(), t.is_empty)
t.push(3)
t.apply_all(5)
print(t.erase_at(0), t.to_list())
t.push(4)
t.erase_range(0..=1)
print(t.to_list())
