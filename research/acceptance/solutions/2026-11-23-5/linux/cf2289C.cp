use std.seq.*

@!multitest
let n: 1..=2e5 = read()
let p: [0..n]1..=n = read()
let lft = p.prev_smaller()
let rgt = p.next_smaller()
var ans = 0
for i in 0..n:
    let a = i - (lft[i] ?? -1)
    let b = (rgt[i] ?? n) - i
    ans += if a == 1 or b == 1: a + b - 1 else: 2
print(ans)
