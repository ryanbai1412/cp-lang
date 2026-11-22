use std.seq.{prev_less, next_less}

let a: [0..6]Int = [2, 1, 5, 6, 2, 3]
let pl = prev_less(a)
let nl = next_less(a)
var best = 0
for i in 0..6:
    let l = pl[i] ?? -1
    let r = nl[i] ?? 6
    best max= a[i] * (r - l - 1)
print(best)

let b: [1..=4]Int = [2, 1, 5, 3]
let pl1 = prev_less(b)
let nl1 = next_less(b)
var best1 = 0
for i in 1..=4:
    let l = pl1[i] ?? 0
    let r = nl1[i] ?? 5
    best1 max= b[i] * (r - l - 1)
print(best1)
