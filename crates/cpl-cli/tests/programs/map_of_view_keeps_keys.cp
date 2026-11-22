// A mapped slice keeps the slice's keys, held lazily or as a list:
// `a[1..3].map(f)[2]` is `f(a[2])` (R04a.9.4).
let a = [10, 20, 30, 40]
let m = a[1..3].map(x => x * 2)
print(m[2], m.keys.first!, m.len)
var w = a[2..].map(x => x + 1)
print(w[3], w.keys.first!)
print(a[1..3].map(x => x * 2)[1])
