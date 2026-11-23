// expect: E0201 at 3
// rule: 11.3.1
// A library module gets no prelude: `kit.prelude_free` calls `gcd` without `use std.math.gcd`.
use kit.prelude_free.f
print(f())
