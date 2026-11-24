// expect: E0320
// rule: 07.1.6
// A range-domain array's keys are `Int`s too: an `Int` variable into its scan is E0320, as on a `List` (decision 507).
let a: [0..3]Int = [5, -3, -2]
let pre = scan(a, max)
for k in 0..=3: print(pre[k]) // <- here
