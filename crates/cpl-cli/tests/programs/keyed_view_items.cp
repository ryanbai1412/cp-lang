// `.keyed().items` counts from 0 on a slice whatever its keys, and
// `.keyed(keys)` counts from the first key of `keys` (R04a.9.6, R12.4.2).
let a = [10, 20, 30, 40]
var s = 0
for (k, x) in a[2..].keyed().items: s += k * x
print(s)
for (k, x) in a[1..3].keyed(5..).items: print(k, x)
let b = a.keyed(1..)
print(b[1], b[4], b.keys.first!)
