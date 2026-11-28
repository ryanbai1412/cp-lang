// expect: ok, output "YES YES YES YES YES YES -inf 0.5000000000 3.0000000000"
// rule: 10.2.6
// input: "0.1 1e-300 2.2250738585072014e-308 2.2250738585072011e-308 9007199254740993 4.9e-324 -0.0 .5 +3e0\n"
// An `F64` token reads as the `F64` nearest it, ties to even, down to the subnormals; `-0.0` keeps its sign.
let (a, b, c, d, e, f, z, h, k): (F64, F64, F64, F64, F64, F64, F64, F64, F64) = read()
print(yes(a == 0.1), yes(b == 1e-300), yes(c == 2.2250738585072014e-308),
    yes(d == 2.225073858507201e-308), yes(e == 9007199254740992.0), yes(f > 0.0 and f / 2.0 == 0.0),
    1.0 / z, h, k)
