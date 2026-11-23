// expect: ok, output "3 YES"
domain V = 0..4
let adj: [V]List<V> = [[V(1)], [V(0), V(2)], [V(1), V(3)], [V(2)]]
let t = Rooted(adj, root: V(0))
let c = Int(t.centroid())
print(t.diameter().len, yes(c == 1 or c == 2))
