// expect: ok, output "2 1"
// rule: R08.1.10
// A closure bound by `let` and then given for a bound that only a fn type's impl meets coerces to
// that fn type, as a lambda written at the bound does.
trait Nx<V>:
    fn deg(self, u: V) -> Int
impl<V, S: Seq<V>> Nx<V> for V -> S:
    fn deg(self, u: V) -> Int = self(u).count()
fn go<V, G: Nx<V>>(g: G, u: V) -> Int = g.deg(u)
let g = [[2, 1], [], [0]]
let f = (u: Int) => g[u]
let h = (u: Int) => [u]
print(go(f, 0), go(h, 5))
