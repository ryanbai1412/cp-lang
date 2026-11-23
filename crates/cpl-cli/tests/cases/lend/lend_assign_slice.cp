// A var declared after its source in the same block may keep a view of it (R05.7.8).
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
var b: [V]Int = 0
b = a[..]
print(b.len)
