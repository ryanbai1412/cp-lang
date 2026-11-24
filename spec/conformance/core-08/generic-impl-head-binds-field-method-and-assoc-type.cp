// expect: ok, output "0->7 0->8 2->7 2->8"
// rule: 08.1.9
// A generic struct's method finds its field's method and associated type through the field type's generic impl: `S<V, H<X, Int>>.out` reads `G.Arc` from `impl Nx<V> for H<V, A>`.
trait Nx<V>:
    type Arc
    fn out(self, u: V) -> yields Self.Arc
struct H<V: Domain, A>(xs: List<A>):
    fn only(self, keep: V -> Bool = v => true) -> S<V, Self> = S(self, keep)
impl Nx<V> for H<V, A>:
    type Arc = A
    fn out(self, u: V) -> yields A:
        for x in xs: yield x
struct S<V, G>(g: G, keep: V -> Bool):
    fn out(self, u: V) -> yields G.Arc:
        if not keep(u): return
        for a in g.out(u): yield a
impl Show for S<V: Domain, G: Nx<V>>:
    fn show(o):
        var s = ""
        for i in 0..V.count:
            let u = V(i)
            for a in o.out(u):
                if not s.is_empty: s += " "
                s += "{u}->{a}"
        s
domain X = 0..3
let h: H<X, Int> = H([7, 8])
print(h.only(keep: v => v != 1))
