// Codeforces 2183 B: compute the smallest missing non-negative integer below K - 1.
@!multitest
let (n, k): (Int, Int) = read()
let a: [0..n]Int = read()
var has: [0..=n]Bool = false
for x in a: has[x] = true
print(first(x for x in 0..k - 1 if not has[x]) ?? (k - 1))
