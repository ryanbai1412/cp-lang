// expect: ok, output "27 27"
// rule: 08.1.5
// A `U64`'s `**` takes an `Int` exponent, as `Pow`'s default right side says, directly and through a bound.
fn cube<T: Pow>(x: T) -> T = x ** 3
let e = 3
print(U64(3) ** e, cube(U64(3)))
