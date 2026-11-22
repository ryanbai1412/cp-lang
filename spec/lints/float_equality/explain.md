Comparing `F64` values with `==` or `!=` is fragile under IEEE-754 rounding. Compare `abs(a - b) < eps` instead.
