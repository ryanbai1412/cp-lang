// Source: Codeforces 2187B (Shortest Statement Ever),
//   https://codeforces.com/contest/2187/problem/B
// limits: 2000 ms, 256 MB
// Problem: given x, y < 2^30, print p, q >= 0 with p & q = 0 minimising |x - p| + |y - q| (any
//   optimal pair). t <= 1e4 tests. Multitest.
// Solution: guess the signs of p - x and q - y (four cases); with them fixed the objective is
//   linear, and a wrong guess only ever overestimates, so the minimum over the cases is exact.
//   Per case a digit DP over bits 30..0 with state (p has left x, q has left y in the guessed
//   direction) tries the bit pairs 00, 01, 10 and keeps the least (cost, p, q).
//   O(4 * 31 * 4 * 3) per test.

fn step(sep: Bool, ge: Bool, v: Bool, bit: Bool) -> ?(Bool, Int):
    if not sep and v != bit and v != ge: return None
    let diff = Int(v) - Int(bit)
    Some((sep or v != bit, if ge: diff else: -diff))

@!multitest
let (x, y): (0..2 ** 30, 0..2 ** 30) = read()
var ans: ?(Int, Int, Int) = None

for ge_x in [false, true]:
    for ge_y in [false, true]:
        var dp: [Bool, Bool]?(Int, Int, Int) = None
        dp[false, false] = (0, 0, 0)
        for b in (0..=30).rev:
            let (bx, by) = (x.bit(b), y.bit(b))
            var ndp: [Bool, Bool]?(Int, Int, Int) = None
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
