@multitest
let n: 1..=100 = read()
let a: [n]Int = read()
var best = a[0]
var run = 0
for x in a:
    run = max(run + x, x)
    best = max(best, run)
print(best)
