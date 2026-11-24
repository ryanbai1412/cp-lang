// expect: ok, output "YES"
// rule: 06.4.3
// A bare `gen()` draws the type its value goes to, the way a generic fn's
// return-only type parameter binds: a declared `let`, each field of a
// `fields` constructor, and a user fn's `T` alike.
trait Zero:
    fn zero() -> Self
impl Zero for Int:
    fn zero() -> Int = 7
fn mk<T: Zero>() -> T = T.zero()
struct P(a: 1..=9, b: 5..=6)
impl Random for P:
    fn gen() -> P = P(gen() for _ in fields(P))
let x: 1..=9 = gen()
let p: P = gen()
let z: Int = mk()
print(yes(x >= 1 and x <= 9 and p.a >= 1 and p.a <= 9 and p.b >= 5 and p.b <= 6 and z == 7))
