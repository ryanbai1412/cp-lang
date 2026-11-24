// `s + x * y` over residues runs as one `%` where the bounds fit: the top residues of a
// modulus near 1e9 and one near 3e9 (where `P - 1 + (P - 1)^2` is just inside an `Int`),
// and a modulus of 2.
use std.mod.{Mod}
type A = Mod<1_000_000_007>
type B = Mod<3_037_000_493>
type C = Mod<2>
let xa = [A(1_000_000_006 - k) for k in 0..50]
let xb = [B(3_037_000_492 - k) for k in 0..50]
let xc = [C(k) for k in 0..50]
var a = A(0)
var b = B(0)
var c = C(0)
for k in 0..50:
    a = a + xa[k] * xa[49 - k]
    b = b + xb[k] * xb[49 - k]
    c = c + xc[k] * xc[49 - k]
print(a, b, c)
let top = xa[0]
print(top + top * top)
