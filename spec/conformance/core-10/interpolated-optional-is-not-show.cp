// expect: E0606 at 3
// rule: 10.3.7
// An optional in an interpolated string is E0606, as in `print` (R10.3.2).
let x: ?Int = Some(3)
print("x={x}")
