// cp: 0.1 aab2e10+dirty
let n: 1..=20 = read()
domain V = 1..=n
let w: [V]1..=100 = read()
var picked = Set<V>()
for v in V:
    if w[v] % 2 == 0:
        picked.insert(v)
var total = 0
for v in picked:
    total += w[v]
print(picked.len, total, picked.min() ?? V.first, picked.max() ?? V.first)
