// expect: ok, output "10 6 4 1"
// rule: std.mod.comb
// `Comb` takes any `Residue` type with a prime modulus, not only `Mod<P>`: its modulus is
// `M.modulus()` and each table entry is `M.reduce(x)`, so a user modular type gets factorials,
// inverses and `choose` with nothing else to write.
use std.mod.{Residue}
struct R13(v: Int)
impl Residue for R13:
    fn modulus() -> Int = 13
    fn residue(self) -> Int = self.v
    fn reduce(x: Int) -> R13 = R13(x % 13)
impl Add for R13:
    fn add(x, y) = R13((x.v + y.v) % 13)
impl Sub for R13:
    fn sub(x, y) = R13((x.v - y.v + 13) % 13)
impl Mul for R13:
    fn mul(x, y) = R13(x.v * y.v % 13)
let c = Comb<R13>()
print(
    c.choose(5, 2).residue(),
    c.fact(3).residue(),
    c.inv(10).residue(),
    c.choose(14, 13).residue(),
)
