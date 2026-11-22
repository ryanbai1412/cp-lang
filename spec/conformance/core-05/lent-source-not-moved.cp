// expect: E0602 at 3
// rule: 05.7.6
let a = [1, 2]
let w = a[0..1]
let b = a
print(w.sum(), b.len)
