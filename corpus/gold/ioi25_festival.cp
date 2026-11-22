// Coupon c turns A tokens into (A − p[c])·t[c]. Exchange argument: multiplying
// coupons (t > 1) are bought in increasing p·t/(t−1). Take them greedily while one
// doesn't shrink A; past that point every multiplier shrinks A, so fewer than 70
// more can ever be afforded: a DP over (prefix, count bought) keeps the most tokens.
// Plain coupons (t = 1) go last, cheapest first.
@grader
fn max_coupons(a0: 0..=1e9, p: [C]1..=1e9, t: [C]1..=4) -> List<C>:
    let cap = 3e14                                   // more than every price combined
    fn buy(a: Int, c: C) -> Int = min((a - p[c]) * t[c], cap)

    let (mult, plain) = C.partition(c => t[c] > 1)
    let mult = mult.sorted(by: c => Ratio(p[c] * t[c], t[c] - 1))
    let plain = plain.sorted(by: c => p[c])

    var a: Int = a0
    var free: List<C> = []
    var rest = mult[..]
    while let c = rest.first and buy(a, c) >= a:
        a = buy(a, c)
        free.push(c)
        rest = rest.drop(1)

    domain K = 0..=min(rest.len, 70)
    var best: [0..=rest.len][K]?Int = none            // most tokens after k of the first i
    best[0][0] = a
    for (i, c) in rest.enumerate():
        for k in K:
            best[i + 1][k] max= best[i][k]
            if let j = k.prev and let t = best[i][j] and t >= p[c]: best[i + 1][k] max= buy(t, c)

    let cost = plain.map(c => p[c]).prefix_sums()
    fn total(k: K) -> Int = k + cost.count(s => s <= best[rest.len][k]!)
    let k = K.filter(k => best[rest.len][k] != none).max(by: total)!

    var took: List<C> = []                            // walk back: a skip that keeps best was optimal
    var j = k
    for i in (0..rest.len).rev:
        if best[i + 1][j] != best[i][j]:
            took.push(rest[i])
            j -= 1
    return free + took.reversed() + plain.take(total(k) - k)
