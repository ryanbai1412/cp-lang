let (h, w): (Int, Int) = read()
domain R = 0..h
domain C = 0..w
domain K = 0..=1
let g: [R, C]Int = read()
var dp: [R, C, K]?Int = none
if g[R(0), C(0)] >= 0:
    dp[R(0), C(0), K(0)] = g[R(0), C(0)]
for r in R:
    for c in C:
        for k in K:
            if let v = dp[r, c, k]:
                if let nr = r.next:
                    if g[nr, c] >= 0:
                        dp[nr, c, k] min= v + g[nr, c]
                    elif let nk = k.next:
                        dp[nr, c, nk] min= v
                if let nc = c.next:
                    if g[r, nc] >= 0:
                        dp[r, nc, k] min= v + g[r, nc]
                    elif let nk = k.next:
                        dp[r, nc, nk] min= v
let a0 = dp[R.last, C.last, K(0)]
let a1 = dp[R.last, C.last, K(1)]
var best: ?Int = a0
if let v1 = a1:
    best min= v1
print(best ?? -1)
