// expect: ok, output "3 4\n4 3\n0\n2 1"
// rule: std.tree Rooted.children
// `children` is a view of the BFS order, so it walks backwards with `.rev` as well as forwards.
domain V = 0..5
let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]
let t = Rooted(adj, root: 0)
print(t.children(1).collect())
print(t.children(1).rev.collect())
print(t.children(2).count())
print([c for c in t.children(0).rev])
