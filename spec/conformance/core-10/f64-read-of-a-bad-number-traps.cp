// expect: R0501 (checked)
// rule: 10.2.6
// input: "1.5x\n"
// An `F64` token is digits with a sign, a `.` and an exponent; `1.5x` is not one, the bad-token trap.
let x: F64 = read() // <- here
print(x)
