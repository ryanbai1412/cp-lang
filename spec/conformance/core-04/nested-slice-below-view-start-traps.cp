// expect: R0101 at 3
// rule: 04a.4.2
let a = [1, 2, 3, 4]
let w = a[2..]
print(w[1..3].len)
