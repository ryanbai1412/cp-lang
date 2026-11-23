domain V = 0..4
let next: [V]List<V> = [[V(1), V(2)], [V(3)], [V(3)], []]
var seen: [V]Bool = false
var q: List<V> = [V(0)]
seen[V(0)] = true
var head = 0
while head < q.len:
    let u = q[head]
    head += 1
    for v in next[u] if not seen[v]:
        seen[v] = true
        q.push(v)
print(q)
