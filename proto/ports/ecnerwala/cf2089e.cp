use lib.math.poly.{Z, convolve}

fn add(a: List<Z>, b: List<Z>) -> List<Z>:
    var sum = a.resized(max(a.len, b.len), 0)
    for i in 0..b.len:
        sum[i] += b[i]
    sum

fn remove(a: List<Z>, b: List<Z>) -> List<Z>:
    if a.is_empty: return []
    convolve(a, b.rev())[b.len - 1..]

@! multitest
let n: Int = read()
var parent: [0..n]?Int = none
for _ in 1..n:
    let (u, v): (Int, Int) = read()
    parent[max(u, v) - 1] = min(u, v) - 1

let comb = Comb<998_244_353>(n)
let inv: [0..=n]Z = [0..=n] k => if k == 0: 0 else: comb.inv_fact(k) * comb.fact(k - 1)

fn integrate(a: List<Z>) -> List<Z>:
    var res: List<Z> = [0]
    for i in 0..a.len:
        res.push(a[i] * inv[i + 1])
    res

fn plus_integral(a: List<Z>) -> List<Z> = add(a, integrate(a))

var sub: [0..n]List<Z> = [0..n] _ => [1]
for v in (0..n).rev:
    sub[v] = plus_integral(sub[v])
    if let p = parent[v]: sub[p] = convolve(sub[p], sub[v])

var before: [0..n]List<Z> = []
var siblings: [0..n]List<Z> = [0..n] _ => [1]
for v in 1..n:
    let p = parent[v]!
    before[v] = siblings[p]
    siblings[p] = convolve(siblings[p], sub[v])

var ans: List<Z> = [1]
var dp: [0..=n]List<Z> = [0..=n] _ => List.filled(n + 1, Z.zero)
dp[n][1] = 1
for t in (1..n).rev:
    dp[t] = add(dp[t], dp[t + 1])
    let stop = if t + 1 < n: parent[t + 1]! else: 0
    var ways: List<Z> = [1]
    var (p, child) = (t, t + 1)
    while p != stop:
        if p != t: ways = convolve(ways, before[child])
        dp[p] = add(dp[p], remove(dp[t + 1], integrate(ways)))
        ways = plus_integral(ways)
        (p, child) = (parent[p]!, p)
    dp[t + 1] = []

    var carry: List<Z> = []
    var node: ?Int = t
    child = t + 1
    while let v = node:
        if v != t: carry = remove(carry, before[child])
        var lifted: List<Z> = [0]
        for i in 0..dp[v].len:
            lifted.push(dp[v][i] * (i + 1))
        let next = add(lifted, carry)
        carry = add(carry, dp[v])
        dp[v] = next
        (node, child) = (parent[v], v)
    ans.push(dp[0][0])
print(ans)
