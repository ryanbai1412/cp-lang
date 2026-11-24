// error: use `.l` / `.r` for the cuts on either side of a position
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l = V(1)
let r = V(2)
print(a[l.r])
