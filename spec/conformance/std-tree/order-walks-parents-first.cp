// expect: ok, output "0 1 2 3 4 5\n6 3 2 1 1 1"
domain V = 0..6
let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0, 5], [1], [1], [2]]
let t = Rooted(adj, root: 0)
print([Int(v) for v in t.order])
var size: [V]Int = 1
for v in t.order.rev:
    if let p = t.parent(v): size[p] += size[v]
print(size)
