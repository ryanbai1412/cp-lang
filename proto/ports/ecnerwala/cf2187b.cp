// Codeforces 2187 B — Minimum Distance Disjoint-Bit Pair
//
// Find non-negative integers `(p, q)` with `p & q == 0` minimizing
// `|p - x| + |q - y|`.
//
// Fixing both comparison directions `(ge_x, ge_y)` in `{false (<=), true (>=)}^2`
// enforces `p >= x` (or `p <= x`) and `q >= y` (or `q <= y`) on every bit
// transition. Because each sign choice is valid only when it matches the true
// comparison, the linear signed objective equals `|p - x| + |q - y|` on every
// reachable pair, and taking the minimum over all four sign choices covers all
// pairs `(p, q)` exactly.
//
// Run a top-down digit DP over bits `30` down to `0` with `[Bool, Bool]` state
// `(sep_x, sep_y)` tracking whether `p` / `q` has already strictly separated
// from `x` / `y` in its chosen direction. One `step(sep, ge, v, bit)` helper
// enforces the direction constraint and returns the updated `(sep', signed_diff)`
// for both axes.

fn step(sep: Bool, ge: Bool, v: Bool, bit: Bool) -> ?(Bool, Int):
    if not sep and v != bit and v != ge: return none
    let diff = Int(v) - Int(bit)
    some((sep or v != bit, if ge: diff else: -diff))

@!multitest
let (x, y): (0..2 ** 30, 0..2 ** 30) = read()
var ans: ?(Int, Int, Int) = none

for ge_x in [false, true]:
    for ge_y in [false, true]:
        var dp: [Bool, Bool]?(Int, Int, Int) = none
        dp[false, false] = (0, 0, 0)
        for b in (0..=30).rev:
            let (bx, by) = (x.bit(b), y.bit(b))
            var ndp: [Bool, Bool]?(Int, Int, Int) = none
            for sx in [false, true]:
                for sy in [false, true] if let (w, px, py) = dp[sx, sy]:
                    for (vx, vy) in [(false, false), (false, true), (true, false)]:
                        if let (nx, dx) = step(sx, ge_x, vx, bx) and let (ny, dy) = step(
                            sy,
                            ge_y,
                            vy,
                            by,
                        ):
                            ndp[nx, ny] min= (w * 2 + dx + dy, px * 2 + Int(vx), py * 2 + Int(vy))
            dp = ndp
        ans min= dp.min()

let (_, p, q) = ans!
print(p, q)
