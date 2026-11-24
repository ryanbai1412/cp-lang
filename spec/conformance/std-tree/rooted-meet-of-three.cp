// expect: ok, output "1 3 2 4"
// rule: std.tree.Rooted.meet
// The meet of three vertices lies on all three paths between them; it is any of them that lies on the other two's path.
domain V = 0..6
let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0, 5], [1], [1], [2]]
let t = Rooted(adj, root: 0)
print(t.meet(3, 4, 5), t.meet(3, 3, 2), t.meet(2, 5, 3), t.meet(1, 4, 4))
