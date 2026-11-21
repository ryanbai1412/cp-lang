// Codeforces 2219 A: factor 2*(P + 2*Q) + 1 = (2*N + 1)*(2*M + 1) and check
// whether Q <= N*M + N for any valid odd factor >= 3.
@!multitest
let (p, q): (Int, Int) = read()
let cnd = (p + 2 * q) * 2 + 1
for i in 3..=isqrt(cnd) by 2 if cnd % i == 0:
    let (n, m) = (i / 2, cnd / i / 2)
    if q <= n * m + n:
        print(n, m)
        return
print(-1)
