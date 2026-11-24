// expect: ok, output "seq 0 seq 1"
use std.lazy.{LazyForest}
var f = LazyForest<Int, Int>(apply: (v, k, g) => v + g, compose: (g, h) => g + h)
let s = f.seq()
let t = f.seq()
print(s, t)
