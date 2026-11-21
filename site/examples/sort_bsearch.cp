let (n, target): (1..=100, Int) = read()
var a: [0..n]Int = read()
a.sort()

var lo = 0
var hi = n
while lo < hi:
    let mid = (lo + hi) / 2
    if a[mid] < target:
        lo = mid + 1
    else:
        hi = mid
print(lo)
