const MOD = 998244353
type Z = Mod<MOD>

fn factor(n0: Int) -> List<(Int, Int)>:
    var n = n0
    var res: List<(Int, Int)> = []
    var d = 2
    while d * d <= n:
        if n % d == 0:
            var e = 0
            while n % d == 0:
                n /= d
                e += 1
            res.push((d, e))
        d += if d == 2: 1 else: 2
    if n > 1: res.push((n, 1))
    res

@!multitest
let (x0, x1, x2): (Int, Int, Int) = read()
var pset: Map<Int, Int> = Map()
for (p, e) in factor(x0): pset[p] = (pset.get(p) ?? 0) + e
for (p, e) in factor(x1): pset[p] = (pset.get(p) ?? 0) + e
for (p, e) in factor(x2): pset[p] = (pset.get(p) ?? 0) + e

var tot: Z = 1
var tot_qs: Map<Int, Int> = Map()
for (p, _) in pset:
    tot *= Z(p - 1)
    for (q, eq) in factor(p - 1):
        tot_qs[q] = (tot_qs.get(q) ?? 0) + eq

tot /= Z(x0)
tot /= Z(x1)
tot /= Z(x2)
for (q, e) in tot_qs:
    let inv_q = Z(q).inv()
    let prob_no_q = inv_q.pow(e)
    let has_q = x0 % q == 0 or x1 % q == 0 or x2 % q == 0
    tot *= prob_no_q + (1 - prob_no_q) * (if has_q: 0 else: inv_q)
print(tot)
