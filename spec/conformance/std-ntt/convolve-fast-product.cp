// expect: ok, output "YES 399 171700 fast 3"
// `convolve`'s product over a `FastProduct` type (NTT for `Mod<P>`) equals the plain loop; a modulus
// without the power of two, and a type that declines (`None`), fall back to the loop. A user type
// joins the same way.
use std.ntt.{convolve, FastProduct}
type Z = Mod<998244353>
type W = Mod<1000000007>
let n = 200
let a: List<Z> = [Z(i * i + 1) for i in 0..n]
let b: List<Z> = [Z(7 * i + 3) for i in 0..n]
var brute = List.filled(2 * n - 1, Z(0))
for i in 0..n:
    for j in 0..n: brute[i + j] += a[i] * b[j]
let c = convolve(a, b)
let aw: List<W> = [W(i + 1) for i in 0..100]
struct Tal(v: Int):
    const zero = Tal(0)
    const one = Tal(1)
impl Add for Tal:
    fn add(x, y) = Tal(x.v + y.v)
impl Sub for Tal:
    fn sub(x, y) = Tal(x.v - y.v)
impl Mul for Tal:
    fn mul(x, y) = Tal(x.v * y.v)
impl FastProduct for Tal:
    fn product(a: List<Tal>, b: List<Tal>) -> ?List<Tal> = if a.len > 1: Some([Tal(-1)]) else: None
let fast = convolve([Tal(1), Tal(2)], [Tal(3)])
let slow = convolve([Tal(1)], [Tal(3)])
print(yes(c == brute), c.len, convolve(aw, aw)[99].v, if fast[0].v == -1: "fast" else: "loop", slow[0].v)
