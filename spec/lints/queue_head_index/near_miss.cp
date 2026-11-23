domain V = 0..4
let next: [V]List<V> = [[V(1), V(2)], [V(3)], [V(3)], []]
let q: List<V> = [V(0)]
var head = 0
while head < q.len:
    head += 1
print(q, head)
var seen: [V]Bool = false
var order: List<V> = [V(0)]
seen[V(0)] = true
var rank: [V]Int = 0
var z = 0
while z < order.len:
    let u = order[z]
    rank[u] = z
    z += 1
    for v in next[u] if not seen[v]:
        seen[v] = true
        order.push(v)
print(rank)
