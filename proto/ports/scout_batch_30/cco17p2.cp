// CCO 2017 P2 Cartesian Conquest: tile an n x m rectangle using 2:1 (or 1:2)
// axis-aligned rectangles, minimizing and maximizing the number of tiles.
// Rethought from 172 lines (custom splitmix64 open-addressed hash table +
// 1005x1005 precomputation table + operator-overloaded monoid): pure Euclidean
// step-jumping DP memoized in HashMap<(Int, Int), (Int, Int)>.
// original: corpus/submissions/6416978.cpp
let n0: 1..=100000000 = read()
let m0: 1..=100000000 = read()

const INF = 1000000000000000000
var memo = HashMap<(Int, Int), (Int, Int)>()

fn combine(a: (Int, Int), b: (Int, Int)) -> (Int, Int) = (min(a.0, b.0), max(a.1, b.1))
fn shift(a: (Int, Int), d: Int) -> (Int, Int) = (a.0 + d, a.1 + d)

fn dp(n: Int, m: Int) -> (Int, Int):
    if n == 0 or m == 0: return (0, 0)
    let jump = if n % 2 == 0: n / 2 else: 2 * n
    if jump > m: return (INF, -INF)
    let key = (n, m)
    if key in memo: return memo[key]
    var ans = (INF, -INF)
    let k_base = max(1, (m - 2 * n) / jump - 1)
    var k = k_base
    while true:
        let nx = m - k * jump
        if nx < 0: break
        let ndp = dp(nx, n)
        if ndp.0 < INF:
            ans = combine(ans, shift(ndp, k))
            if n % 2 == 0:
                let cntk = (k / 4) + (k % 4)
                ans = combine(ans, shift(ndp, cntk))
        k += 1
    memo[key] = ans
    ans

let res = combine(dp(n0, m0), dp(m0, n0))
print(res.0, res.1)
