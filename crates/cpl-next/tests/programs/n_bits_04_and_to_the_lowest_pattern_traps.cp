// expect: R0202
// rule: R01.5.2, Int is -(2^63-1)..=2^63-1
// The two values' `&` is -2^63, which is not an `Int`.
let a = -9223372036854775807
print(a & -2) // <- here
