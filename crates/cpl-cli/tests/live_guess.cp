// Finds a number from 1 to 100 by asking the judge, which answers one
// question at a time.
var lo = 1
var hi = 100
while lo < hi:
    let mid = (lo + hi + 1) / 2
    print("?", mid)
    let answer: Str = read()
    if answer == "<": hi = mid - 1
    else: lo = mid
print("!", lo)
