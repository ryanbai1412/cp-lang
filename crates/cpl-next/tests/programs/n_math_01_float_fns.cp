// expect: ok, output "-3 -2 -3 -2\n3 2 3 2\n3 -3\n1.4142135624 5.0000000000 0.7853981634\n2.0000000000 1.0000000000"
// std.math's `F64` fns are intrinsics both engines and the C++ compute the C library's way, an
// `Int` argument taken as the `F64` it is (R02.11.1); `round` takes a half away from zero.
let x = -2.5
print(floor(x), ceil(x), round(x), trunc(x))
let y = 2.5
print(ceil(y), floor(y), round(y), trunc(y))
print(round(2.6), round(-2.6))
print(sqrt(2), hypot(3, 4), atan2(1, 1))
print(sqrt(4.0), ln(exp(1.0)))
