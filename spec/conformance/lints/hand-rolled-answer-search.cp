// expect: L0520
let a = [1, 3, 5, 7]
var (lo, hi) = (0, a.len)
while lo < hi: // <- here
    let mid = (lo + hi) / 2
    if a[mid] >= 5: hi = mid else: lo = mid + 1
print(lo)
