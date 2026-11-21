const MOD = 998244353
type Z = Mod<MOD>
let comb = Comb<Z>()

fn place_ways(bins: Int, balls: Int) -> Z = comb.choose(bins - 1 + balls, balls)
fn place_weight(bins: Int, balls: Int) -> Z = if balls <= 0: Z(0) else: comb.choose(
    bins - 1 + balls,
    bins,
)

@!multitest
let n: Int = read()
let a: [0..n]Z = read()

var pref: [0..=n]Z = 0
var pref_i: [0..=n]Z = 0
for i in 0..n:
    pref[i + 1] = pref[i] + a[i]
    pref_i[i + 1] = pref_i[i] + a[i] * i

var ans = pref[n] * n
for l in 2..=n:
    let extras = n - l
    let k = extras / 2
    let mid_w = if extras % 2 == 1:
        place_ways(l, k) * 2 + place_weight(l, k) * 4
    else:
        place_ways(l, k) + place_ways(l, k - 1) + (place_weight(l, k) + place_weight(l, k - 1)) * 2
    let edge_w = mid_w + place_ways(l, if extras % 2 == 1: k else: k - 1)

    ans += edge_w * (pref[n - l + 1] + pref[n] - pref[l - 1])
    let cap = min(l - 2, n - l + 1)
    let incr = pref_i[cap] + (pref[n - cap] - pref[cap]) * cap + (pref[n] - pref[n - cap]) * (n - 1) - (pref_i[n] - pref_i[n - cap])
    ans += mid_w * incr

print(ans)
