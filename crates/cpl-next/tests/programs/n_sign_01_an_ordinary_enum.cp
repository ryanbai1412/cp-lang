// expect: ok, output "-1 0 1 1 -1 0\nYES 0 1 -1\nYES YES"
// `Sign` is an ordinary enum: `Int(s)` by its `Into<Int>`, `-s` by its `Neg`, and `Neg < Zero < Pos`
// in declaration order, so `max(s, Sign.Zero)` clamps it.
let a = (-5).sign
let z = 0.sign
let p = 7.sign
print(Int(a), Int(z), Int(p), Int(-a), Int(-p), Int(-z))
print(yes(a < z and z < p), Int(max(a, Sign.Zero)), Int(max(p, Sign.Zero)), Int(min(a, p)))
print(yes(-a == Sign.Pos), yes((-5).sign == Sign.Neg))
for x in [-9, 0, 4]: assert x == Int(x.sign) * abs(x)
