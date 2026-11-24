// expect: ok, output "3 4 2 4"
// rule: 8.1.2
// One generic `reach` walks a table of vertices, a table of records and a fn,
// each through its shape impl of `Nx`, and reads each arc through `Tg`.
trait Tg<V>:
    @get fn tgt(self) -> V
impl<V: Domain> Tg<V> for V:
    @get fn tgt(self) -> V = self
impl<V, R: (to: V, ..)> Tg<V> for R:
    @get fn tgt(self) -> V = self.to
trait Nx<V>:
    type Arc: Tg<V>
    fn outs(self, u: V) -> yields Self.Arc
impl<V: Domain, S: Seq<A>, A: Tg<V>> Nx<V> for [V]S:
    type Arc = A
    fn outs(self, u: V) -> yields A:
        for a in self[u]: yield a
impl<V: Domain, S: Seq<A>, A: Tg<V>> Nx<V> for V -> S:
    type Arc = A
    fn outs(self, u: V) -> yields A:
        for a in self(u): yield a
fn reach<V: Domain, G: Nx<V>>(g: G, s: V) -> Int:
    var seen: [V]Bool = false
    var st = [s]
    seen[s] = true
    var n = 0
    while let u = st.pop():
        n += 1
        for a in g.outs(u):
            let v = a.tgt
            if not seen[v]:
                seen[v] = true
                st.push(v)
    n
domain V = 0..4
let g: [V]List<V> = [[1], [2], [], [0]]
let h: [V]List<(to: V, w: Int)> = [[(to: 1, w: 3)], [], [], []]
let f = (u: V) => [V.wrap(u.ord + 1)]
print(reach(g, V(0)), reach(g, V(3)), reach(h, V(0)), reach(f, V(0)))
