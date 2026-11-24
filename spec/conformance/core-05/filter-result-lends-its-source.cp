// expect: E0602
// rule: 05.5.3
// `filter(xs, p)` is a `Filtered` holding `xs`, so `xs` is lent to `f` while `f` lives (Consumers report).
use std.seq.{filter}
var xs = [1, 2, 3]
let f = filter(xs, x => x > 1)
xs.push(9) // <- here
print(f.count(x => x > 0))
