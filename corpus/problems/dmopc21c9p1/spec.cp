// dmopc21c9p1: neighbouring positive numbers sum to a[i]; count the first numbers that work.
// Each x is x1 or -x1 plus a constant, so positivity bounds x1 from below or above in turn.
let n = read<2..=1_000_000>()
var sign = 1
var c = 0
var ok = 1..
for a in read<[n - 1]Int>():
    sign = -sign
    c = a - c
    ok = ok.clip(if sign < 0: ..c else: 1 - c..)
print(ok.len)
