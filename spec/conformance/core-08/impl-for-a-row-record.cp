// expect: ok, output "3 2"
// rule: 8.1.2
// An impl for every record with a `to` field is found for `(to: V, w: Int)`,
// beside a blanket impl for the domain itself, also through a generic bound.
trait Tg<V>:
    @get fn tgt(self) -> V
impl<V: Domain> Tg<V> for V:
    @get fn tgt(self) -> V = self
impl<V, R: (to: V, ..)> Tg<V> for R:
    @get fn tgt(self) -> V = self.to
trait Nx<V>:
    type Arc: Tg<V>
    fn outs(self, u: V) -> yields Self.Arc
fn t<V, A: Tg<V>>(a: A) -> V = a.tgt
domain V = 0..4
let r: (to: V, w: Int) = (to: 2, w: 1)
print(t(V(3)).ord, t(r).ord)
