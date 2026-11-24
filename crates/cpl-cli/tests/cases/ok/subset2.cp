// cp: 0.1 5cd4a99+dirty
// Set algebra is in place, as on `Bitset`: copy first when both sides are needed again.
let n: 1..=20 = read()
domain V = 1..=n
let w: [V]1..=100 = read()
var even = Set<V>()
var big = Set<V>()
for v in V:
    if w[v] % 2 == 0: even.insert(v)
    if w[v] > 5: big.insert(v)
var either = even.clone()
either.union(big)
var both = even.clone()
both.intersect(big)
var only = even.clone()
only.subtract(big)
print(either.len, both.len, only.len)
