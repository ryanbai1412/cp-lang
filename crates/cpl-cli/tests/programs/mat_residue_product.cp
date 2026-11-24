// A `Mat` product over residues sums products unreduced and reduces once per run of steps: a
// modulus near 1e9 with rows long enough to reduce mid-row, the 2^31 edge, one past it (the
// plain loop), and moduli 2 and 1.
use std.mod.{Mod}
use std.matrix.{Mat}
fn top<P>(n: Int) -> Mat<Mod<P>>:
    Mat<Mod<P>>.from_flat(n, n, [Mod<P>(P - 1 - (i % 3)) for i in 0..n * n])
fn sum<P>(m: Mat<Mod<P>>) -> Mod<P>:
    var s = Mod<P>(0)
    for i in 0..m.rows:
        for j in 0..m.cols: s += m.get(i, j)
    s
let a = top<1_000_000_007>(40)
print(sum(a * a))
let b = top<2_147_483_648>(12)
print(sum(b * b))
let c = top<2_147_483_649>(12)
print(sum(c * c))
let d = top<2>(9)
print(sum(d * d))
let e = top<1>(5)
print(sum(e * e))
let f = Mat<Mod<998_244_353>>.from_flat(2, 3, [Mod(1), Mod(2), Mod(3), Mod(4), Mod(5), Mod(6)])
let g = Mat<Mod<998_244_353>>.from_flat(3, 1, [Mod(7), Mod(8), Mod(9)])
let h = f * g
print(h.get(0, 0), h.get(1, 0))
