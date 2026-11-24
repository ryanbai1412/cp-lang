// expect: E0606
// rule: std.derive.hash_fields
// std has no `impl Hash for F64`, so a tuple holding one is not `Hash` either.
fn ok<T: Hash>(a: T) -> Int = 1
print(ok((1, 2.5)))  // <- here
