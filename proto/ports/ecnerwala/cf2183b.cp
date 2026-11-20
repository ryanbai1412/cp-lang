// Codeforces 2183 B: compute the smallest missing non-negative integer below K - 1.
@!multitest
let (n, k): (Int, Int) = read()
let a: [0..n]Int = read()
var has: [0..=n]Bool = false
for i in 0..n: has[a[i]] = true
var ans = 0
while ans < k - 1 and has[ans]: ans += 1
print(ans)
