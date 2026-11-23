let n: 1..=200000 = read()
domain V = 1..=n
let a: [V]1..=n = read()
var pos: Map<V, Int> = Map()  // expect: domain_keyed_map
for v in V:
    pos[v] = a[v]
print(pos.len)
