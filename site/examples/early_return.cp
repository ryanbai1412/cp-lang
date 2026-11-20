let n: 1..=100 = read()
domain V = 1..=n
let a: [V]Int = read()

if n == 1:
    print(0)
    return

var lo = a[V.first]
var hi = a[V.first]
for v in V:
    lo min= a[v]
    hi max= a[v]
print(hi - lo)
