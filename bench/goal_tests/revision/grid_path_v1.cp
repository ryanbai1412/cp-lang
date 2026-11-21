let (h, w): (Int, Int) = read()
domain R = 0..h
domain C = 0..w
let g: [R, C]Int = read()
var dp: [R, C]?Int = none
if g[R(0), C(0)] >= 0:
    dp[R(0), C(0)] = g[R(0), C(0)]
for r in R:
    for c in C:
        if g[r, c] < 0:
            continue
        if let v = dp[r, c]:
            if let nr = r.next:
                if g[nr, c] >= 0:
                    dp[nr, c] min= v + g[nr, c]
            if let nc = c.next:
                if g[r, nc] >= 0:
                    dp[r, nc] min= v + g[r, nc]
print(dp[R.last, C.last] ?? -1)
