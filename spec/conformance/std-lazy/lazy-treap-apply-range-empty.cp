// expect: ok, output "1 1 6"
use std.lazy.{LazyTreap}
var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f)
t.push(1)
t.push(1)
t.push(1)
t.apply_range(1, 1, 9)
t.apply_range(2, 3, 5)
print(t.to_list())
