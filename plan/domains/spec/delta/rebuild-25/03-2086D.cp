// Codeforces 2086D Even String: https://codeforces.com/problemset/problem/2086/D
// Each letter lives wholly on odd or wholly on even positions. Any split whose odd letters fill the
// ceil(S/2) odd slots gives odd! * even! / prod c! strings, the same for every split, so the answer
// is (number of such splits) * odd! * even! / prod c!. Letters with c = 0 are not choices.
type Z = Mod<998244353>
let comb = Comb<Z>(5e5)

@!multitest
let c = read<[26]0..=5e5>()
let total = c.sum()
let odd = (total + 1) / 2
var ways: [0..=odd]Z = 0
ways[0] = 1
for x in c if x > 0:
    for s in (x..=odd).rev:
        ways[s] += ways[s - x]
print(ways[odd] * comb.fact(odd) * comb.fact(total - odd) * c.map(x => comb.inv_fact(x)).prod())
