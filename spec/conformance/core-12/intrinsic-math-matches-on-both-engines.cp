// expect: ok, output "3 7 2.5000000000 1.5000000000 2.0000000000 -3 -2 -3 -2 7 4"
// rule: 12.14.3
// std's math fns are `@intrinsic` bodies: each reads as a call or a getter, keeps its `Int`, `I128` and `F64` forms, and rounds to an `Int` the same way on both engines.
let a = [-3, 4]
let f = [-2.5, 2.25]
let big: I128 = -7
let r = round
print(a[0].abs, abs(big) + abs(a[1]) - 4, f[0].abs, f[1].sqrt, sqrt(a[1]), f[0].floor, ceil(f[0]), f[0].round, trunc(f[0]), abs(big), r(3.5))
