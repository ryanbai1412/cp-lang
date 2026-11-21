const MOD = 998244353
type Z = Mod<MOD>

@!multitest
let n: Int = read()
let a: [0..n]Z = read()
let comb = Comb<MOD>()

var pref: [0..=n]Z = 0
var pref_i: [0..=n]Z = 0
for i in 0..n:
    pref[i + 1] = pref[i] + a[i]
    pref_i[i + 1] = pref_i[i] + a[i] * i

fn place_ways(bins: Int, balls: Int) -> Z:
    if balls < 0: return 0
    comb.fact(bins - 1 + balls) * comb.inv_fact(balls) * comb.inv_fact(bins - 1)

fn place_weight(bins: Int, balls: Int) -> Z:
    if balls <= 0: return 0
    comb.fact(bins - 1 + balls) * comb.inv_fact(balls - 1) * comb.inv_fact(bins)

var ans: Z = 0
for i in 0..n: ans += a[i] * n

for l in 2..=n:
    let extras = n - l
    var mid_w: Z = 0
    var edge_w: Z = 0
    if extras % 2 == 1:
        let k = (extras - 1) / 2
        mid_w = place_ways(l, k) * 2 + place_weight(l, k) * 4
        edge_w = mid_w + place_ways(l, k)
    else:
        let k = extras / 2
        mid_w = place_ways(l, k) + place_ways(l, k - 1) + place_weight(l, k) * 2 + place_weight(
            l,
            k - 1,
        ) * 2
        edge_w = mid_w + place_ways(l, k - 1)
    ans += edge_w * (pref[n - l + 1] - pref[0])
    ans += edge_w * (pref[n] - pref[l - 1])
    let cap = min(l - 2, n - l + 1)
    let incr = pref_i[cap] + (pref[n - cap] - pref[cap]) * cap + (pref[n] - pref[n - cap]) * (n - 1) - (pref_i[n] - pref_i[n - cap])
    ans += mid_w * incr
print(ans)
