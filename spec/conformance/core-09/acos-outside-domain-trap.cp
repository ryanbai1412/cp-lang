// expect: R0401 (checked)
// `acos` traps outside -1..=1 instead of giving NaN, as `asin` does.
// input: "-1.5\n"
let x: F64 = read()
print("{acos(x):.6f}") // <- here
