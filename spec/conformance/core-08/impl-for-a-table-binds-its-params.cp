// expect: ok, output "2"
// rule: 8.1.2
// An impl for a table type with its own params, `[V]List<A>`, is found for a
// `[V]List<V>` argument: selection binds `V` and `A` from the argument.
trait Tg<V>:
    @get fn tgt(self) -> V
impl<V: Domain> Tg<V> for V:
    @get fn tgt(self) -> V = self
impl<V, R: (to: V, ..)> Tg<V> for R:
    @get fn tgt(self) -> V = self.to
trait Nx<V>:
    type Arc: Tg<V>
    fn outs(self, u: V) -> yields Self.Arc
impl<V: Domain, A> Nx<V> for [V]List<A>:
    type Arc = A
    fn outs(self, u: V) -> yields A:
        for a in self[u]: yield a
fn cnt<V: Domain, G: Nx<V>>(g: G, s: V) -> Int:
    var n = 0
    for a in g.outs(s): n += 1
    n
domain V = 0..4
let g: [V]List<V> = [[1, 2], [2], [], [0]]
print(cnt(g, V(0)))
