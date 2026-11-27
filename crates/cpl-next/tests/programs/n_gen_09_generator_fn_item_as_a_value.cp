// expect: ok, output "26 3 2"
// rule: R07.6.2
// A `-> yields T` fn named as a value is a fn whose call is a generator value: `h(3)` walks
// `ng(3)` from the start each time, and the item meets a bound through a fn type's impl.
fn ng(n: Int) -> yields Int:
    for i in 0..n: yield i * 2
trait Nx<V>:
    fn out(self, u: V) -> Int
impl<V, S: Seq<V>> Nx<V> for V -> S:
    fn out(self, u: V) -> Int = self(u).count()
fn go<V, G: Nx<V>>(g: G, u: V) -> Int = g.out(u)
let h = ng
var s = 0
for x in h(3): s += x
for x in ng(2): s += x * 10
let g = h(2)
print(s, go(ng, 3), g.count())
