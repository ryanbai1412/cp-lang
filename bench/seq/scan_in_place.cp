let n = 200_000
var v: List<Int> = [0 for _ in 0..n]
var seed = 123456789
for i in 0..n:
    seed = (seed * 1103515245 + 12345) % 2147483647
    v[i] = (seed % 2001) - 1000

var acc = 0
for r in 0..400:
    let lo = v.scan(min)
    for i in 0..n: v[i] = lo.prefix(i + 1)
    acc += v[n - 1 - r]
    let hi = [x for x in v.rev].scan(max)
    for i in 0..n: v[i] = hi.prefix(n - i)
    acc += v[r]
    v[r] = r - 1000

print(acc)
