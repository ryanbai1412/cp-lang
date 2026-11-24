// Many small trees given by parent arrays, as a multitest reads them: `Rooted.from_parents`
// builds each one, and the checksum is every vertex's 3 * depth + subtree size (cf2138c1).
@!multitest
let n: 2..=1000 = read()
domain V = 1..=n
let raw: [2..=n]V = read()
var par: [V]?V = None
for i in 2..=n: par[V(i)] = raw[i]
let t = Rooted.from_parents(par)
var s = 0
for v in V: s += 3 * t.depth(v) + t.size(v)
print(s)
