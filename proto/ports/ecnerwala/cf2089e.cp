use lib.math.poly.{Z, convolve}

fn poly_remove(a: List<Z>, b: List<Z>) -> List<Z>:
    if a.is_empty:
        return []
    var tmp = b
    tmp.reverse()
    let prod = convolve(a, tmp)
    let drop = b.len - 1
    var res: List<Z> = []
    for i in drop..prod.len:
        res.push(prod[i])
    res

fn poly_add(a: List<Z>, b: List<Z>) -> List<Z>:
    var res = a
    while res.len < b.len:
        res.push(0)
    for i in 0..b.len:
        res[i] += b[i]
    res

@! multitest
let n: Int = read()
var par: [0..n]Int = -1
for _ in 0..n - 1:
    let (u0, v0): (Int, Int) = read()
    let u = min(u0, v0) - 1
    let v = max(u0, v0) - 1
    par[v] = u
var fact: [0..=n]Z = 1
var fcur: Z = 1
for i in 1..=n:
    fcur *= i
    fact[i] = fcur
var ifact: [0..=n]Z = 1
ifact[n] = fact[n].inv()
var fi = n
while fi >= 1:
    ifact[fi - 1] = ifact[fi] * fi
    fi -= 1
var invs: [0..=n]Z = 0
for k in 0..n:
    invs[k + 1] = ifact[k + 1] * fact[k]

fn poly_integ(a: List<Z>) -> List<Z>:
    var res: List<Z> = [Z(0)]
    for i in 0..a.len:
        res.push(a[i] * invs[i + 1])
    res

fn poly_or_integ(a: List<Z>) -> List<Z>:
    var res = a
    res.push(0)
    var i = res.len - 1
    while i > 0:
        res[i] += res[i - 1] * invs[i]
        i -= 1
    res

var sub_ways: [0..n]List<Z> = []
for i in 0..n:
    sub_ways[i] = [Z(1)]
var bi = n - 1
while bi >= 0:
    sub_ways[bi] = poly_or_integ(sub_ways[bi])
    if bi > 0:
        let p = par[bi]
        sub_ways[p] = convolve(sub_ways[p], sub_ways[bi])
    bi -= 1
var left_sib: [0..n]List<Z> = []
var tmp_sib: [0..n]List<Z> = []
for i in 0..n:
    tmp_sib[i] = [Z(1)]
for i in 1..n:
    let p = par[i]
    left_sib[i] = tmp_sib[p]
    tmp_sib[p] = convolve(tmp_sib[p], sub_ways[i])
var ans: List<Z> = [Z(1)]
var dp: [0..=n]List<Z> = []
for i in 0..=n:
    dp[i] = List.filled(n + 1, Z(0))
dp[n][1] = 1
var t = n - 1
while t > 0:
    dp[t] = poly_add(dp[t], dp[t + 1])
    let a = if t + 1 < n: par[t + 1] else: 0
    var cur_ways: List<Z> = [Z(1)]
    var p = t
    var nc = t + 1
    while p != a:
        if p != t:
            cur_ways = convolve(cur_ways, left_sib[nc])
        dp[p] = poly_add(dp[p], poly_remove(dp[t + 1], poly_integ(cur_ways)))
        cur_ways = poly_or_integ(cur_ways)
        nc = p
        p = par[p]
    dp[t + 1] = []
    var cw: List<Z> = []
    var p2 = t
    var nc2 = t + 1
    while p2 >= 0:
        if p2 != t:
            cw = poly_remove(cw, left_sib[nc2])
        let nlen = max(cw.len, dp[p2].len + 1)
        var ndp: List<Z> = [Z(0)]
        for i in 0..dp[p2].len:
            ndp.push(dp[p2][i] * (i + 1))
        while ndp.len < nlen:
            ndp.push(0)
        ndp = poly_add(ndp, cw)
        cw = poly_add(cw, dp[p2])
        dp[p2] = ndp
        nc2 = p2
        p2 = par[p2]
    ans.push(dp[0][0])
    t -= 1
print(ans)
