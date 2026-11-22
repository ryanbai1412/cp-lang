domain V = 1..=3
let a: [V]Int = [3, 1, 2]
let _ = V.sort(by: a)
let _ = V.first(v => a[v] > 1)
