// expect: ok, output "3 4 9 31622 3037000499"
// `isqrt` is std.math's body, reached by a call, `isqrt(n)`, or as a property, `n.isqrt` (a `@get` takes no `()`, R06.4.2): no compiler special case.
let n = 1_000_000_007
print(isqrt(15), isqrt(16), 99.isqrt, n.isqrt, isqrt(9_223_372_036_854_775_807))
