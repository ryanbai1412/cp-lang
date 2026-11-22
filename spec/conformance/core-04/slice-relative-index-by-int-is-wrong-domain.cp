// expect: E0301 at 5
// rule: 04a.5.3
domain V = 1..=5
let a: [V]Int = [V] v => Int(v)
let l = V.first!
let k = 2
print(a[l..][k])
