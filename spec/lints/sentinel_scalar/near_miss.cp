let n: 1..=100 = read()
var sign = -1
for i in 0..n:
    sign = -sign
var (lo, hi) = (0, 10**18)
while hi - lo > 1:
    let mid = (lo + hi) / 2
    if mid * mid <= n: lo = mid else: hi = mid
print(sign, lo)
