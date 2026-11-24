// expect: ok, output "6 3 7 -4"
// rule: R08.1.5
// An impl fn's parameters and result written with no type take the ones
// its trait declares, with `Self` and the trait's arguments put in: a user
// trait's `fn scale(self, k: Int) -> Self` makes `fn scale(self, k)` take an
// `Int` and give a `P`, and `impl Mul<Int> for P` makes `fn mul(a, k)` take a
// `P` and an `Int`.
trait Scale:
    fn scale(self, k: Int) -> Self
    fn both(a: Self, b: Self) -> Self
struct P(v: Int)
impl Scale for P:
    fn scale(self, k) = P(self.v * k)
    fn both(a, b) = P(a.v + b.v)
impl Add for P:
    fn add(a, b) = P(a.v + b.v)
impl Mul<Int> for P:
    fn mul(a, k) = P(a.v * k)
impl Neg for P:
    fn neg(a) = P(-a.v)
impl Eq for P:
    fn eq(a, b) = a.v == b.v
print(P(2).scale(3).v, P.both(P(1), P(2)).v, (P(3) + P(2) * 2).v, (-P(4)).v)
assert(P(1) + P(1) == P(2))
