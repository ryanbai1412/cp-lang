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
            let (bx, by) = (Int(x.bit(b)), Int(y.bit(b)))
            var ndp: [0..2, 0..2]?(Int, Int, Int) = none
            for cx in 0..2:
                for cy in 0..2 if let (w, px, py) = dp[cx, cy]:
                    for vx in 0..2:
                        for vy in 0..2 if vx + vy < 2:
                            if cx == 0 and vx != bx and Int(vx > bx) != dx: continue
                            if cy == 0 and vy != by and Int(vy > by) != dy: continue
                            let nx = Int(cx == 1 or vx != bx)
                            let ny = Int(cy == 1 or vy != by)
                            let nw = w * 2 + (if dx == 1: vx - bx else: bx - vx) + (
                                if dy == 1: vy - by else: by - vy
                            )
                            ndp[nx, ny] min= (nw, px * 2 + vx, py * 2 + vy)
            dp = ndp
        ans min= dp.min()

let (_, p, q) = ans!
print(p, q)
