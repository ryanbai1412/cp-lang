// expect: E0402
// A punned `xs:` given to a `mut` parameter is not marked `mut`, so it is the same error as `xs: xs`.
fn grow(mut xs: List<Int>):
    xs.push(1)
var xs = [1, 2]
grow(xs:)
print(xs)
