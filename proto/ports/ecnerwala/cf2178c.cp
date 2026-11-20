@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var best = a[0]
var cur = 2 * a[0]
for i in 1..n:
    best max= cur + a[i]
    cur max= cur + 2 * a[i]
print(best - sum(a))
