// expect: ok, output "1"
// rule: 07.3.1
// `Rooted.up` folds `U` values that `lift` makes from each child's `T`, so `T` and `U` may differ: here a win flag lifts to a count of losing children (rebuild round 29 L2).
domain V = 0..3
let adj: [V]List<V> = [[1, 2], [0], [0]]
let t = Rooted(adj, root: 0)
let win = t.up(+, lift: (w, c, p) => if w: 0 else: 1, finish: (losing, v) => losing > 0)
print(sum(1 for v in V if win[v]))
