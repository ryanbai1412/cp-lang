// error: is mutably lent to `part`
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l = V(1)
let r = V(2)
let part = a[l..=r]
part.sort()
print(a[l])
part.reverse()
