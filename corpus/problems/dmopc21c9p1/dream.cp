// dmopc21c9p1: neighbouring positive numbers sum to a[i]; count the first numbers that work.
// Each x is x1 or -x1 plus a constant, so positivity bounds x1 from below or above in turn.
let n = read<2..=1_000_000>()
var sign = 1
var c = 0
var lo = 1
var hi = inf
for a in read<[n - 1]Int>():
    sign = -sign
    c = a - c
    if sign < 0:
        hi min= c - 1
    else:
        lo max= 1 - c
print(max(hi - lo + 1, 0))
