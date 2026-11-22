// expect: ok, output "1 2 0"
use std.lazy.{LazyTreap}
var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f)
t.push(1)
t.push(2)
var u = t.split_at(2)
t.append(mut u)
print(t.to_list(), u.len)
