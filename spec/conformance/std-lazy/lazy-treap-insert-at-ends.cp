// expect: ok, output "0 1 2"
use std.lazy.{LazyTreap}
var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f)
t.insert_at(0, 1)
t.insert_at(0, 0)
t.insert_at(2, 2)
print(t.to_list())
