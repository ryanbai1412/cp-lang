// expect: ok, output "1 2 2 3"
use std.lazy.{LazyTreap}
var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f, before: (x, y) => x < y)
t.insert(2)
t.insert(3)
t.insert(1)
t.insert(2)
print(t.to_list())
