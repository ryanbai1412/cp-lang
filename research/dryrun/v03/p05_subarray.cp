// CSES 1660 Subarray Sums I: count windows of positive a with sum exactly x.
let (n, x): (1..=2e5, 1..=1e9) = read()
domain I = [n]
let a = read<[I]1..=1e9>()
var count = 0
var sum = 0
var l = I.cuts.first!
for r in I:
    sum += a[r]
    while sum > x:
        sum -= a[l.r!]
        l = l.next!
    if sum == x: count += 1
print(count)
