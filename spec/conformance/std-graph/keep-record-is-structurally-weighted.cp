// expect: ok, output "4"
// A `keep:` record with `.to` and `.w` is structurally `Target<V>` and `Weighted<W>`, so `matrix()` takes it (R02.5.2; rebuild round 6 H-KEEP-RECORD).
domain V = 0..3
let es: [0..2](u: V, v: V, w: 1..=9) = [(u: 0, v: 1, w: 4), (u: 1, v: 2, w: 5)]
let g = Graph.undirected(es, keep: r => (to: r.to, w: Int(r.w)))
let d = g.matrix()
print(d[V(0), V(1)]!)
