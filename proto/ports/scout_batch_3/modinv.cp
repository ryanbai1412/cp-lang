// Modular inverse: given x and m with gcd(x, m) = 1, both below 2^64, print
// the y in 0..m with x * y = 1 (mod m).
use std.math.{ext_gcd}                                              // WISH F1

let x: U64 = read()                                                 // WISH G9: inputs past Int
let m: U64 = read() where gcd(x, m) == 1
// x * s + m * t = 1. `%` is Euclidean, so a negative s needs no fix-up.
let (_, s, _) = ext_gcd(I128(x), I128(m))                           // WISH G9: wide intermediates
print(s % m)
