// Codeforces 2147G: Modular Tetration, tuned.
// port.cp's product, factoring each p - 1 straight off `sieve.spf` into the exponent map
// instead of building a `factor` list per prime.

const MOD = 998_244_353
type Z = Mod<MOD>

let sieve = Sieve(10 ** 6)

@!multitest
let xs: [0..3]1..=10 ** 6 = read()
let pset = [p for x in xs for (p, _) in sieve.factor(x)].sorted().deduped()

var tot: Z = 1
var tot_qs: Map<1..=10 ** 6, Int> = Map()
for p in pset:
    tot *= p - 1
    var rest = p - 1
    while rest > 1:
        let q = sieve.spf[rest]
        while rest % q == 0:
            rest /= q
            tot_qs[q] += 1

for x in xs: tot /= x
for (q, e) in tot_qs:
    let inv_q = Z(q).inv()
    let prob_no_q = inv_q ** e
    let has_q = pset.contains(q)
    tot *= prob_no_q + (1 - prob_no_q) * (if has_q: 0 else: inv_q)
print(tot)
