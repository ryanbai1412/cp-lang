// expect: ok, output "3 1"
// rule: 06.1.5
// `or` and `and` are bare operators like `+`: `each(c: +, t: or)` is the field-wise monoid of a count and a flag (spec/lib/rationale/trees.md's `reroot(each(count: +, taken: or), ...)`).
use std.algebra.{Monoid, each}
let m = Monoid<(c: Int, t: Bool)>.of(each(c: +, t: or))
let r = m.op((c: 1, t: false), (c: 2, t: true))
print(r.c, Int(r.t))
