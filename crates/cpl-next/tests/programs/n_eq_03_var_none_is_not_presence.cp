// expect: E0606
// rule: R08.1.5
// A `var` holds the `None` value only until it is assigned, so comparing with one is
// `Optional`'s `Eq`, which needs `T: Eq`; a function type has none.
fn inc(x: Int) -> Int = x + 1
let c: ?(Int -> Int) = inc
var n: ?(Int -> Int) = None
if c == n: print(1) else: print(0) // <- here
