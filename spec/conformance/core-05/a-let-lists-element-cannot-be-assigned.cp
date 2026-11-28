// expect: E0401
// rule: 05.6.3
// `xs[0] = 7` writes the cell `xs` lends, and a `let` list lends none: the fix is `var xs`.
let xs: List<Int> = [1]
xs[0] = 7  // <- here
print(xs[0])
