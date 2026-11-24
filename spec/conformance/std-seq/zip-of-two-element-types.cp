// expect: ok, output "1 a\n2 b\n1 x\n2 y\n1 4"
// rule: 06.1.5
// `zip`, `zip_exact` and `product` take each side's element type from that side, so the two
// sides may hold different types (`List<Int>` with a `Str`).
use std.seq.{zip_exact}
for (n, c) in zip([1, 2, 3], "ab"): print(n, c)
for (n, c) in zip_exact([1, 2], "xy"): print(n, c)
print(product([1], [4]).collect()[0])
