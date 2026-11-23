let (n, s): (1..=2e5, 1..=1e18) = read()
let a: [0..n]1..=1e9 = read()
var best = 0
var sum = 0
var lo = 0
for hi in 0..n:
    sum += a[hi]
    while sum > s:
        sum -= a[lo]
        lo += 1
    best max= hi - lo + 1
print(best)
