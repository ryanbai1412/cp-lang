// Codeforces 2187 B — Minimum Distance Disjoint-Bit Pair
//
// Find non-negative integers `(p, q)` with `p & q == 0` minimizing
// `|p - x| + |q - y|`.
//
// Fix the two comparison signs `dx in {0 (p <= x), 1 (p >= x)}` and
// `dy in {0 (q <= y), 1 (q >= y)}` so the objective is linear in the bits of
// `p` and `q`. Run a top-down digit DP over bits `30` down to `0` with state
// `(cx, cy)` tracking whether `p` / `q` has already separated from `x` / `y`
// in the chosen direction, choosing `(vx, vy)` with `vx + vy < 2` (`p & q == 0`).

@!multitest
let (x, y): (0..2 ** 30, 0..2 ** 30) = read()
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
