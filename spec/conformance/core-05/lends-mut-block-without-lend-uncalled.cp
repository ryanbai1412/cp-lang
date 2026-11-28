// expect: E0604
// rule: 05.6.3
// engines: next
// The lender rule holds on the fn as written, called or not: a block that ends in a plain place lends nothing.
fn first(mut xs: List<Int>) -> lends mut Int:
    let k = 0
    xs[k] // <- here
print(1)
