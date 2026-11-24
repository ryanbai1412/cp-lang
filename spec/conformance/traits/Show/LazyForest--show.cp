// expect: ok, output "1 sequences"
use std.lazy.{LazyForest}
var f = LazyForest<Int, Int>(apply: (v, k, g) => v + g, compose: (g, h) => g + h)
let s = f.seq()
print(f)
