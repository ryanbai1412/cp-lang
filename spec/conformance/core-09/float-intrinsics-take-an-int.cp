// expect: ok, output "0.841471 1.000000 1.557408 1.570796 1.570796 0.785398 5.000000 2.718282 0.000000"
// The `F64` intrinsics of std.math take an `Int` as well, like `sqrt`: each is a std fn whose
// body is `@intrinsic`, so an `Int` argument reads as the `F64` it is.
let n = 1
let z = 0
print("{sin(n):.6f} {cos(z):.6f} {tan(n):.6f} {asin(n):.6f} {acos(z):.6f} {atan2(n, n):.6f} {hypot(3, 4):.6f} {exp(n):.6f} {ln(n):.6f}")
