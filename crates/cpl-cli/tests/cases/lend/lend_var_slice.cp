// A var or container declared after its source in the same block may keep a view of it (R05.7.8).
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l = V(1)
let r = V(2)
var keep = a[l..=r]
print(keep.len)
