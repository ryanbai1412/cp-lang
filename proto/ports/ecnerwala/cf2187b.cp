// Codeforces 2187 B: bitwise top-down DP over the 4 sign combinations of (p - x, q - y)
// minimizing |p - x| + |q - y| subject to p & q == 0.
@!multitest
let (x, y): (Int, Int) = read()
var ans: ?(Int, Int, Int) = none

for dx in 0..2:
    for dy in 0..2:
        var dp: [0..2, 0..2]?(Int, Int, Int) = none
        dp[0, 0] = (0, 0, 0)
        for b in (0..=30).rev:
            let bx = Int(x.bit(b))
            let by = Int(y.bit(b))
            var ndp: [0..2, 0..2]?(Int, Int, Int) = none
            for cx in 0..2:
                for cy in 0..2:
                    if let (w, px, py) = dp[cx, cy]:
                        for vx in 0..2:
                            for vy in 0..2:
                                if vx == 1 and vy == 1: continue
                                if cx == 0 and vx != bx and Int(vx > bx) != dx: continue
                                if cy == 0 and vy != by and Int(vy > by) != dy: continue
                                let nx = Int(cx == 1 or vx != bx)
                                let ny = Int(cy == 1 or vy != by)
                                let nw = w * 2 + (if dx == 1: vx - bx else: bx - vx) + (if dy == 1: vy - by else: by - vy)
                                let cand = (nw, px * 2 + vx, py * 2 + vy)
                                if let prev = ndp[nx, ny]:
                                    if nw < prev.0: ndp[nx, ny] = cand
                                else:
                                    ndp[nx, ny] = cand
            dp = ndp
        for cx in 0..2:
            for cy in 0..2:
                if let v = dp[cx, cy]:
                    if let cur = ans:
                        if v.0 < cur.0: ans = v
                    else:
                        ans = v

let (_, p, q) = ans!
print(p, q)
