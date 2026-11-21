domain V = read(1..=5)
let a: [V]Int = read()
var s = 0
for v in V: s += a[v] * v.ord
let k = read(0..10)
print(V.count, s, k)
fn sub(m: Int) -> Int:
    domain W = m
    var t: [W]Int = 1
    sum(t)
print(sub(4))
