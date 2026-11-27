// expect: ok, output "1\n3\n2\n5"
// rule: R08.1.2
// A fn value meets an impl for a fn type (`impl<V, S> Walk<V> for V -> S`) with its own types:
// the impl's `S` is the fn's result, so walking it yields the fn's values in both engines.
domain V = 0..4
trait Walk<V>:
    fn walk(self, u: V)
impl<V: Domain, S: Seq<V>> Walk<V> for V -> S:
    fn walk(self, u: V):
        for a in self(u): print(Int(a))
fn go<W: Domain, N: Walk<W>>(n: N, u: W):
    n.walk(u)
go((p: V) => [V(1), V(3)], V(0))
let h: V -> List<V> = p => [V(2)]
go(h, V(0))
domain R = 0..2
domain C = 0..3
go((p: (R, C)) => [(R(1), C(2))], (R(0), C(0)))
