// expect: E0608 at 3
// rule: 12.11.4
let a = (xs: [1], k: 2)
let b = (..a, k: 3)
print(a.xs.len)
