// p16 with a hand-written lower bound (row 13).
let n: 1..=2e5 = read()
let a = read<[n]1..=1e9>()
var tails: List<Int> = []
for v in a:
    var lo = 0
    var hi = tails.len
    while lo < hi:
        let mid = (lo + hi) / 2
        if tails[mid] >= v: hi = mid else: lo = mid + 1
    if lo == tails.len: tails.push(v) else: tails[lo] = v
print(tails.len)
