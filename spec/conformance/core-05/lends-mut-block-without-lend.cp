// expect: E0604
// rule: 05.6.3
// engines: next
// A block body lends with `lend mut place`; one that ends in a plain place lends nothing.
fn first(mut xs: List<Int>) -> lends mut Int:
    let k = 0
    xs[k] // <- here
var a = [1, 2]
first(mut a) = 9
print(a[0])
