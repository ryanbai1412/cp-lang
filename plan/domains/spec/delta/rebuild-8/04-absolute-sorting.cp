// 1772D Absolute Sorting
// in: t; n; a_1..a_n. out: per test, any x in 0..=1e9 that makes |a_i - x| non-decreasing, or -1.
@!multitest
let n = read<2..=2e5>()
let a = read<[n]1..=1e8>()
var lo = 0
var hi = 1e9
for (x, y) in a.adjacent:
    if x < y: hi min= (x + y) / 2
    elif x > y: lo max= ceil_div(x + y, 2)
print(if lo <= hi: lo else: -1)
