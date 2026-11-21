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
let a: [0..n]Int = read()

let pref = prefix_n([Z(x) for x in a])
let pref_i = prefix_n([Z(a[i]) * i for i in 0..n])

var ans = pref.prefix(n) * n
for l in 2..=n:
    let extras = n - l
    let k = extras / 2
    let mid_w = if extras % 2 == 1:
        place_ways(l, k) * 2 + place_weight(l, k) * 4
    else:
        place_ways(l, k) + place_ways(l, k - 1) + (place_weight(l, k) + place_weight(l, k - 1)) * 2
    let edge_w = mid_w + place_ways(l, if extras % 2 == 1: k else: k - 1)

    ans += edge_w * (pref.prefix(n - l + 1) + pref.sum(l - 1..n))
    let cap = min(l - 2, n - l + 1)
    let incr = pref_i.prefix(cap) + pref.sum(cap..n - cap) * cap + pref.sum(n - cap..n) * (n - 1) - pref_i.sum(n - cap..n)
    ans += mid_w * incr

print(ans)
