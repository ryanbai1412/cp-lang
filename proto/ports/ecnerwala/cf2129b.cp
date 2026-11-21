@!multitest
let n: Int = read()
let p: List<Int> = [read::<Int>() for _ in 0..n]
var ans = 0
for i in 0..n:
    let lo = count(p[j] > p[i] for j in 0..i)
    let hi = count(p[j] > p[i] for j in (i + 1)..n)
    ans += min(lo, hi)
print(ans)
