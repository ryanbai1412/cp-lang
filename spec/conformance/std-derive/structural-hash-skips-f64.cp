// expect: E0606
// rule: 06.4.14
// `@structural` `Hash` reaches a scalar only through its own impl, and `F64` has none.
fn ok<T: Hash>(a: T) -> Int = 1
print(ok(1.5)) // <- here
