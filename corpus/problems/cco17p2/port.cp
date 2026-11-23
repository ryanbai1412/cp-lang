// CCO 2017 P2 Cartesian Conquest (CP Lang 0.2.2): Euclidean step-jumping DP
// using Tropical Optionals `(?Int, ?Int)` with `min=`/`max=` (zero `INF` sentinel).
// original: corpus/submissions/6416978.cpp
let (n0, m0): (1..=100_000_000, 1..=100_000_000) = read()

var memo = Map<(Int, Int), (?Int, ?Int)>()

fn dp(n: Int, m: Int) -> (?Int, ?Int):
    if n == 0 or m == 0: return (0, 0)
    let jump = if n % 2 == 0: n / 2 else: 2 * n
    if jump > m: return (None, None)
    let key = (n, m)
    if let hit = memo.get(key): return hit
    var (lo, hi): (?Int, ?Int) = (None, None)
    var k = max(1, (m - 2 * n) / jump - 1)
    while m - k * jump >= 0:
        let (nlo, nhi) = dp(m - k * jump, n)
        lo min= nlo + k
        hi max= nhi + k
        if n % 2 == 0:
            let cntk = (k / 4) + (k % 4)
            lo min= nlo + cntk
            hi max= nhi + cntk
        k += 1
    memo[key] = (lo, hi)
    (lo, hi)

let (a_lo, a_hi) = dp(n0, m0)
let (b_lo, b_hi) = dp(m0, n0)
var best_lo = a_lo
best_lo min= b_lo
var best_hi = a_hi
best_hi max= b_hi
print(best_lo!, best_hi!)
