// expect: ok, output "0.000000 1.000000 1.000000 1.570796 3.141593"
// `sin`, `cos`, `tan`, `asin` and `acos` take and give `F64` radians, like `atan2`.
let q = atan2(1.0, 1.0)
print("{sin(0.0):.6f} {cos(0.0):.6f} {tan(q):.6f} {asin(1.0):.6f} {acos(-1.0):.6f}")
