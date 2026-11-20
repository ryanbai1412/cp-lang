@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var ans = -max(a)!
for i in 0..n:
    ans += max(a[i], a[(i + 1) % n])
print(ans)
