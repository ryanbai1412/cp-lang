let (n, m, q) = read<(1..=2e5, 1..=2e5, 1..=2e5)>()
domain V = 1..=n
domain Street = 1..=m
let street = read<[Street](V, V)>()
var deg: [V]Int = 0
for (u, v) in street:
    deg[u] += 1
    deg[v] += 1
for closed in read<[q]Range<Street>>():
    print(deg[street[closed.first!].0], closed.len)
