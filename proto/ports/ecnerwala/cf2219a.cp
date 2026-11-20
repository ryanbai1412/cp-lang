// Codeforces 2219 A: factor 2*(P + 2*Q) + 1 = (2*N + 1)*(2*M + 1) and check
// whether Q <= N*M + N for any valid odd factor >= 3.
@!multitest
let (p, q): (Int, Int) = read()
let cnd = (p + 2 * q) * 2 + 1
var found: ?(Int, Int) = none
var i = 3
while i * i <= cnd:
    if cnd % i == 0:
        let n = i / 2
        let m = (cnd / i) / 2
        if q <= n * m + n:
            found = (n, m)
            break
    i += 2
if let (n, m) = found: print(n, m)
else: print(-1)
