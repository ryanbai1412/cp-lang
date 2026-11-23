// expect: ok, output "YES"
domain V = 0..3
let adj: [V]List<V> = [[V(1), V(2)], [V(0)], [V(0)]]
let t = Rooted(adj, root: V(0))
print(yes(t.farthest(Some((len: 0, u: V(1), v: V(1))), None) == Some((len: 0, u: V(1), v: V(1)))))
