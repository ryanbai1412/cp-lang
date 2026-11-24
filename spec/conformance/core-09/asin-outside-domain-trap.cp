// expect: R0001 (checked)
// `asin` and `acos` trap outside -1..=1 instead of giving NaN.
// input: "2\n"
let x: F64 = read()
print("{asin(x):.6f}") // <- here
