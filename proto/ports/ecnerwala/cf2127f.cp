type Z = Mod<1_000_000_007>
let t: Int = read()
var cases: List<(Int, Int)> = []
var lim = 2
for _ in 0..t:
    let (n, m): (Int, Int) = read()
    cases.push((n, m))
    lim max= n + m + 2
var fact: [0..(lim + 1)]Z = 1
for i in 1..=lim:
    fact[i] = fact[i - 1] * i
var ifact: [0..(lim + 1)]Z = 1
ifact[lim] = fact[lim].inv()
for i in (1..=lim).rev:
    ifact[i - 1] = ifact[i] * i
fn choose(n: Int, r: Int) -> Z:
    if r < 0 or r > n: return 0
    fact[n] * ifact[r] * ifact[n - r]
for (n, m) in cases:
    var ans: Z = 0
    let inv_n1 = Z(n - 1).inv()
    let inv_n2: Z = if n >= 3: Z(n - 2).inv() else: 0
    for mx in 0..=m:
        var cnt: Z = 0
        var c = 0
        while mx + c * (mx + 1) <= m:
            let term = choose(n - 1, c) * choose(m - mx - c * (mx + 1) + (n - 2), n - 2)
            if (c & 1) == 1: cnt -= term else: cnt += term
            c += 1
        ans += cnt * (Z(mx) - Z(m - mx) * inv_n1)
        if n >= 3 and 2 * mx <= m:
            var cnt2: Z = 0
            var c2 = 0
            while 2 * mx + c2 * (mx + 1) <= m:
                let term = choose(n - 2, c2) * choose(m - 2 * mx - c2 * (mx + 1) + (n - 3), n - 3)
                if (c2 & 1) == 1: cnt2 -= term else: cnt2 += term
                c2 += 1
            ans += cnt2 * (Z(mx) - Z(m - 2 * mx) * inv_n2) * Z(n - 2)
    print(ans)
