// expect: ok, output "7\n4\n3 4\n3 2"
// A row bound `R: (f: T, ..)` takes any record with those fields, in any order, and reads
// them as members; an impl for every such record beats a blanket one for a domain (R08.4.1).
fn span<R: (lo: Int, hi: Int, ..)>(r: R) -> Int = r.hi - r.lo
print(span((lo: 2, hi: 9)))
print(span((hi: 5, name: "x", lo: 1)))
fn weights<V, R: (w: V, ..)>(xs: List<R>) -> List<V> = [x.w for x in xs]
print(weights([(w: 3, to: 1), (w: 4, to: 2)]))
trait Tg<V>:
    @get fn tgt(self) -> V
impl<V: Domain> Tg<V> for V:
    @get fn tgt(self) -> V = self
impl<V, R: (to: V, ..)> Tg<V> for R:
    @get fn tgt(self) -> V = self.to
fn t<V, A: Tg<V>>(a: A) -> V = a.tgt
domain V = 0..4
let r: (to: V, w: Int) = (to: 2, w: 1)
print(t(V(3)).ord, t(r).ord)
