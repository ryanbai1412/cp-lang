// expect: E0303
// rule: diagnostics E0303
// Wrong argument count.
fn add(a: Int, b: Int) -> Int = a + b
print(add(1))  // <- here
