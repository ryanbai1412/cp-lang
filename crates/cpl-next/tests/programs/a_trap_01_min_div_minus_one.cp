// expect: R0202
// rule: R01.5.2, Int is -(2^63-1)..=2^63-1
// input: "-9223372036854775807\n"
let n: Int = read()
let m = (n - 1) / -1  // <- here
print(m)
