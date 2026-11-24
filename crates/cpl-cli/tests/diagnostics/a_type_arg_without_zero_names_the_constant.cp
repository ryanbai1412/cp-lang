use std.algebra.{Monoid}
struct Low(c: Int)
fn (a: Low) + (b: Low) -> Low = Low(min(a.c, b.c))
let m = Monoid<Low>.of(+)
