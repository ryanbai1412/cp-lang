// expect: ok, output "2\n1"
// rule: 08.3
// A static call binds the struct's own params through a chain of bounds: `S: Indexed<E, R>` gives R, and `R: (u: V, v: V, ..)` gives V.
struct G<V>(n: Int):
    fn make<E, R: (u: V, v: V, ..), S: Indexed<E, R>>(edges: S) -> List<V>:
        [edges.at(e).v for e in edges.keys]
domain V = 0..3
domain E = 0..2
let es: [E](u: V, v: V) = [(u: V(0), v: V(1)), (u: V(1), v: V(2))]
print(G<V>.make(es).len)
print(G.make(es)[0])
