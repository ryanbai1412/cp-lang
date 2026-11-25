// expect: R0202
// rule: Int is -(2^63-1)..=2^63-1
// Negating the smallest Int: -(-(2^63-1)) - ... Int is symmetric so neg never overflows; MAX+1 does.
let m = 9223372036854775807
let a = -m - 1  // <- here
print(a)
