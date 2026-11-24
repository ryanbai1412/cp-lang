// expect: ok, output "1 2\n0"
// rule: std.algebra.MinCount
// `MinCount` mirrors `MaxCount`: `+` is its `join` and `zero` its identity, so a fold takes `+` alone.
use std.algebra.{MinCount}
let m = [3, 1, 1].map(v => MinCount.of(v)).fold(+)
print(m.val, m.count)
print(MinCount<Int>.zero.count)
