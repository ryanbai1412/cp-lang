// A `?Cost` field with `Cost = 0..=1e7` is stored in 4 bytes, read and written as any `?Int`.
type Cost = 0..=1e7
struct Node:
    var best: ?Cost
    at: Int
var nodes = [Node(Some(10_000_000), 1), Node(None, 2)]
nodes[1].best = Some(7)
nodes[0].best = None
let total = sum((n.best ?? 0) + n.at for n in nodes)
print(nodes[0].best ?? -1, nodes[1].best ?? -1, total)
