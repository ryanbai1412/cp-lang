// p05 on Int indices (row 5).
let (n, x): (1..=2e5, 1..=1e9) = read()
let a = read<[n]1..=1e9>()
var count = 0
var sum = 0
var l = 0
for r in 0..n:
    sum += a[r]
    while sum > x:
        sum -= a[l]
        l += 1
    if sum == x: count += 1
print(count)
