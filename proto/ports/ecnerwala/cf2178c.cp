// Codeforces 2178 C: pick a split point where earlier values contribute
// positively (if positive) and later values contribute negatively.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()

var best = a[0]
var cur = 2 * a[0]
for x in a[1..n]:
    best max= cur + x
    cur += max(0, 2 * x)
print(best - sum(a))
