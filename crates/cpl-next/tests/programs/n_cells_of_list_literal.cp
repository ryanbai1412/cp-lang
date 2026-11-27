// expect: ok, output "1 9 7\n2 3\n0 2"
// rule: R04.8.10 R02.5.2
// A list literal for `[D]List<E>` whose elements cannot be `E`s gives one list per cell, each
// element then taking `E`, so a literal part is a position; one whose elements can is a fill.
domain V = 0..2
let g: [V]List<V> = [[1], [0, 1]]
let h: [V]List<(to: V, lab: Int)> = [[(to: 1, lab: 7)], [(to: 0, lab: 9)]]
print(Int(g[V(0)][0]), h[V(1)][0].lab, h[V(0)][0].lab)
let f: [V]List<Int> = [4, 5]
print(f[V(1)].len, g[V(1)].len + 1)
var e: [V]List<V> = []
e[V(1)].push(V(0))
print(e[V(0)].len, e[V(1)].len + 1)
