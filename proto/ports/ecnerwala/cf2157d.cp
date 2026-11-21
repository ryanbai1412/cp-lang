// Codeforces 2157 D: pair smallest with largest after sorting, clamping the
// endpoints to [l, r] and summing every positive contribution.
@!multitest
let (n, l, r): (Int, Int, Int) = read()
let raw: [0..n]Int = read()
let a = sorted(raw)
print(sum(max(0, max(l, a[n - 1 - i]) - min(r, a[i])) for i in 0..n))
