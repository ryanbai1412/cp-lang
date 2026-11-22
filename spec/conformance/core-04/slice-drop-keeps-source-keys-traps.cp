// expect: R0101 at 3
// rule: 04a.9.1
let a = [10, 20, 30]
let w = a[..].drop(1)
print(w[0])
