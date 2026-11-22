// p04 with a hand-written binary search (row 3).
let (n, t): (1..=2e5, 1..=1e9) = read()
let k = read<[n]1..=1e9>()
fn enough(time: Int) -> Bool:
    var made = 0
    for x in k:
        made += time / x
        if made >= t: return true
    false
var lo = 1
var hi = 1e18
while lo < hi:
    let mid = lo + (hi - lo) / 2
    if enough(mid): hi = mid else: lo = mid + 1
print(lo)
