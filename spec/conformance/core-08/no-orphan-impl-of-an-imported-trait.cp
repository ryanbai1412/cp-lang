// expect: E0501 at 3
// rule: 08.5.2
// A trait brought in by `use` is not declared here, so this is an orphan.
use std.algebra.{Monoid}
impl Monoid for F64:
    fn one() -> F64 = 0.0
