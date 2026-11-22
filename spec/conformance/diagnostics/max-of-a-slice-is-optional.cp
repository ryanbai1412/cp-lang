// expect: E0302 at 5
let n: 1..=5 = read()
domain V = 1..=n
let a: [V]Int = read()
let f = V.first!
let k: Int = a[f..f].max()
