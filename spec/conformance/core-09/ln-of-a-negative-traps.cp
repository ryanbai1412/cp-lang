// expect: R0401 (checked)
// `ln` below zero traps instead of giving NaN; `ln(0.0)` is `-inf`, as in C++.
// input: "-2\n"
let x: F64 = read()
print("{ln(x):.6f}") // <- here
