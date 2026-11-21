use std.math.{factor}

const MOD = 998244353
type Z = Mod<MOD>

@!multitest
let xs: [0..3]Int = read()
var pset: Set<Int> = Set()
for x in xs:
    for (p, _) in factor(x):
        pset.insert(p)

var tot: Z = 1
var tot_qs: Map<Int, Int> = Map()
for p in pset:
    tot *= p - 1
    for (q, eq) in factor(p - 1):
        tot_qs[q] += eq

for x in xs: tot /= x
for (q, e) in tot_qs:
    let inv_q = Z(q).inv()
    let prob_no_q = inv_q ** e
    let has_q = any(x % q == 0 for x in xs)
    tot *= prob_no_q + (1 - prob_no_q) * (if has_q: 0 else: inv_q)
print(tot)
