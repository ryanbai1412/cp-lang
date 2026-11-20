@!multitest
let (n, l, r): (Int, Int, Int) = read()
var a: [0..n]Int = read()
a.sort()
var ans = 0
for i in 0..n:
    let lo = min(r, a[i])
    let hi = max(l, a[n - 1 - i])
    if lo >= hi: break
    ans += hi - lo
print(ans)
