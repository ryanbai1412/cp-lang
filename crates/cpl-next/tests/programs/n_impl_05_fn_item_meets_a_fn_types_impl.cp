// expect: ok, output "2 1 3"
// rule: R08.1.10
// A fn item given for a type parameter whose bound only a fn type's impl meets coerces to that
// fn type, as a lambda does: `nb` meets `Nx<Int>` through `impl .. for V -> S`.
trait Nx<V>:
    fn out(self, u: V) -> Int
impl<V, S: Seq<V>> Nx<V> for V -> S:
    fn out(self, u: V) -> Int = self(u).count()
fn go<V, G: Nx<V>>(g: G, u: V) -> Int = g.out(u)
fn nb(p: Int) -> List<Int> = [p, p]
fn one(p: Int) -> List<Int> = [p]
print(go(nb, 3), go(one, 3), go(p => [p, p, p], 3))
