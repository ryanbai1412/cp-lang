// error: only when `i != j`
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l = V(1)
let r = V(2)
swap(mut a[l], mut a[l])
