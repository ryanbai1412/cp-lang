// expect: E0603
// rule: 05.7.8
// Returning the view a type parameter bound from a fn not declared `lends` would copy it out of sight; the fix says to write `copy` or declare `lends`.
fn id<T>(x: T) -> T = x // <- here
let a = [1, 2, 3, 4]
print(id(a[1..3]))
