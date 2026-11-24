// expect: E0606
// rule: 07.7.12
// `product` walks `b` once per element of `a`, so `b` must be `Indexed`; a generator runs once and can't restart.
use std.seq.{product}
fn g() -> yields Int:
    yield 1
    yield 2
print(product([1], g()).count()) // <- here
