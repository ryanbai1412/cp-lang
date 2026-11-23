// expect: ok, output "2.718282 0.000000 5.000000 1.000000"
// `exp`, `ln` and `hypot` take and give `F64`, like `sqrt`; `hypot(x, y)` is the length of `(x, y)`.
let e = exp(1.0)
print("{e:.6f} {ln(1.0):.6f} {hypot(3.0, 4.0):.6f} {ln(e):.6f}")
