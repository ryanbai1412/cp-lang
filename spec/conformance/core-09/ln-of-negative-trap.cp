// expect: R0001 at 5 (checked)
// `ln` traps on a negative number instead of giving NaN.
// input: "-1\n"
let x: F64 = read()
print("{ln(x):.6f}")
