// error: this assignment would keep the lent slice
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l = V(1)
let r = V(2)
var b: [V]Int = 0
b = a[l..=r]
