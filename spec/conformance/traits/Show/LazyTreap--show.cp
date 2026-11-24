// expect: ok, output "6 7"
use std.lazy.{LazyTreap}
var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f)
t.push(1)
t.push(2)
t.apply_all(5)
print(t)
