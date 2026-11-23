use std.seq.{prev_smaller, next_smaller}
@!multitest
let n: 1..=2e5 = read()
let p: [0..n]1..=2e5 = read()
let l = p.prev_smaller()
let r = p.next_smaller()
var total = 0
for i in 0..n:
    let a = i - (l[i] ?? -1)
    let b = (r[i] ?? n) - i
    total += if min(a, b) == 1: a + b - 1 else: 2
print(total)
