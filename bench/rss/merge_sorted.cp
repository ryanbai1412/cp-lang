let n: 1..=1_000_000 = read()
domain I = 0..n
let a: [I]Int = read()
let b: [I]Int = read()
var (last, bad, sum) = (-1, 0, 0)
for x in a.merge_sorted(b):
    if x < last: bad += 1
    last = x
    sum += x
print(bad, sum)
