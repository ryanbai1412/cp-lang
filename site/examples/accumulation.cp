let n: 1..=100 = read()
domain V = 1..=n
let a: [V]0..=1000 = read()
var total = 0
for v in V:
    total += a[v]
print(total)
