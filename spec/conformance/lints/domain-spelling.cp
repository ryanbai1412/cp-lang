// expect: L0512 at 3
domain V = 1..=3
let a: [V]Int = [3, 1, 2]
let _ = V.sorted(by: a)
