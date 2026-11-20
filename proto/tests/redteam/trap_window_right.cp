let n: 1..=5 = read()
domain V = 1..=n
var a: [V]Int = read()
let z = V.last
let k: Int = read()
print(sum(a[z.l..z.r]))
print(sum(a[z.l..z.r(k)]))
