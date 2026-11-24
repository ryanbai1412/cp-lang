// expect: R0401
// rule: 08.1.2
// `split_at` past a sequence's length traps instead of splitting another sequence's nodes.
use std.lazy.{LazyForest}
var f = LazyForest<Int, Int>(apply: (v, k, g) => v + g, compose: (g, h) => g + h)
let s = f.seq()
f.push(s, 1)
let u = f.split_at(s, 2)  // <- here
print(f.len(u))
