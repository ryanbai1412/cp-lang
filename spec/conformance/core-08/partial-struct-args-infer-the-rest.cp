// expect: ok, output "1 2 1 2"
// rule: 08.3
// `G<V>.mk(..)` writes the struct's first argument; the rest come from the call, as they
// do for `G.mk(..)`.
struct G<V: Domain, A>:
    arcs: List<A>
    fn mk<S: Indexed<E, R>, E, R: (u: V, ..), B>(edges: S, keep: R -> B = r => r) -> G<V, B>:
        G<V, B>([keep(edges[e]) for e in edges.keys])
domain V = 0..3
let es = [(u: V(0), w: 2)]
let g = G<V>.mk(es)
let h = G<V>.mk(es, keep: r => r.w)
let i = G.mk(es)
print(g.arcs.len, h.arcs[0], i.arcs.len, G<V>.mk(es, keep: r => r.w).arcs[0])
