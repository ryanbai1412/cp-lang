// n: 2048
// fixed: P, k
use std.poly.{Poly}
type Z = Mod<998_244_353>
let n: Int = read()
var acc = 0
let c = [Z(i * 7 + 1) for i in 0..n]
let p = Poly(c)
var zc = [Z(i * 3 + 2) for i in 0..n]
zc[0] = Z(0)
let z = Poly(zc)
for _ in 0..512:
    acc ^= p.deriv().len // timed: poly.Poly.deriv
    acc ^= p.integral().len // timed: poly.Poly.integral
    acc ^= p.eval(Z(3)).v // timed: poly.Poly.eval
for i in 0..1 << 17:
    acc ^= Poly<Z>.from_int(i).len // timed: poly.Poly.from_int
    acc ^= Poly<Z>.zero.len // timed: poly.Poly.zero
    acc ^= Poly<Z>.one.len // timed: poly.Poly.one
    acc ^= Poly<Z>.x.len // timed: poly.Poly.x
    acc ^= p.len // timed: poly.Poly.len
    acc ^= p.deg()! // timed: poly.Poly.deg
    acc ^= if p.is_zero(): 1 else: 0 // timed: poly.Poly.is_zero
    acc ^= p.at(i % n).v // timed: poly.Poly.at
var q = Poly(c)
for i in 0..1 << 17:
    q.set(i % n, Z(i)) // timed: poly.Poly.set
for _ in 0..12:
    acc ^= p.inv(n).len // timed: poly.Poly.inv
    acc ^= p.log(n).len // timed: poly.Poly.log
    acc ^= z.exp(n).len // timed: poly.Poly.exp
    acc ^= p.pow(5, n).len // timed: poly.Poly.pow
    acc ^= p.sqrt(n)!.len // timed: poly.Poly.sqrt
for _ in 0..4096:
    acc ^= p.truncated(n / 2).len // timed: poly.Poly.truncated
    acc ^= Poly.new(c).len // timed: poly.Poly.new
for _ in 0..32:
    acc ^= p.transposed_mul(q).len // timed: poly.Poly.transposed_mul
for _ in 0..2:
    acc ^= p.shift(Z(3)).len // timed: poly.Poly.shift
print(acc, q.at(1).v)
