// expect: R0202
// rule: Int is -(2^63-1)..=2^63-1
// abs-like: m * -1 fine; (m) * 2 overflows; check `-x` of result of sub hitting MIN.
let m = 9223372036854775807
let b = 0 - m
let c = b - 1  // <- here
print(c)
