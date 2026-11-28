// expect: E0902
// rule: 04.4.24
// A constant argument outside a range-typed parameter is rejected when compiled; a module-level
// `let` that keeps a constant is one.
fn half(x: 0..=100) -> Int = x / 2
let n: Int = 200
print(half(n * 1)) // <- here
