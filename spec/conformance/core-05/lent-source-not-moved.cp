// expect: E0602
// rule: 05.7.6
let a = [1, 2]
let w = a[0..1]
let b = a // <- here
print(w.sum(), b.len)
