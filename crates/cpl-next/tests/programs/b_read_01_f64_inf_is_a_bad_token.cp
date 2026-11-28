// expect: R0501
// rule: R10.2.6
// input: "inf\n"
// `inf` is not an `F64` token, which is digits with a sign, a `.` and an exponent: the bad-token trap.
let x: F64 = read() // <- here
print(x)
