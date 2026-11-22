// cp: 0.1 5cd4a99+dirty
let n: 1..=20 = read()
domain V = 1..=n
let w: [V]1..=100 = read()
var even = Set<V>()
var big = Set<V>()
for v in V:
    if w[v] % 2 == 0: even.insert(v)
    if w[v] > 5: big.insert(v)
print((even | big).len, (even & big).len, (even - big).len)
