// expect: E0604
// A `lends mut` fn lends one place on every path through it: its one expression, or a
// `lend mut place` in a block. A block ending in a plain place lends nothing, so the
// caller's write would be lost (R05.6.3).
fn first(mut xs: List<Int>) -> lends mut Int:
    let k = 0
    xs[k] // <- here
var a = [1, 2]
first(mut a) = 9
print(a[0])
