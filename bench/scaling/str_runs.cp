// n: 512
// fixed: A
use std.strings.{lyndon_runs, lcs_all}
let n: Int = read()
let ss = [Str([Char(Int('a') + (i * 7919 + i / 3) % 3) for i in 0..n]) for _ in 0..2]
let ws = [Str([Char(Int('a') + (i * 7 + j) % 2) for i in 0..n / 4]) for j in 0..4]
var acc = 0
for r in 0..256:
    let s = ss[r % 2]
    acc += lyndon_runs(s).len // timed: strings.lyndon_runs
    acc += lcs_all<Char, Str>(ws) // timed: strings.lcs_all
print(acc)
