// expect: E0320
// rule: 07.1.6
// `a.scan(max)` on a range-domain array types as `scan(a, max)`: an `Int` variable into it is E0320 (decision 507).
let a: [0..3]Int = [5, -3, -2]
let pre = a.scan(max)
for k in 0..=3: print(pre[k]) // <- here
