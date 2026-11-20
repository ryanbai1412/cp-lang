// error: `.push(...)` would keep the lent slice
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l = V(1)
let r = V(2)
var parts: List<[V]Int> = []
parts.push(a[l..=r])
