type Z = Mod<1_000_000_007>
let comb = Comb<Z.MOD>()

fn ways(k: Int, rem: Int, mx: Int) -> Z:
    var res: Z = 0
    for c in 0..=min(k, rem / (mx + 1)):
        let term = comb.choose(k, c) * comb.multichoose(rem - c * (mx + 1), k)
        if c % 2 == 1: res -= term else: res += term
    res

@! multitest
let (n, m): (Int, Int) = read()
let inv_n1 = Z(n - 1).inv
let inv_n2: Z = if n >= 3: Z(n - 2).inv else: 0

var ans: Z = 0
for mx in 0..=m:
    ans += ways(n - 1, m - mx, mx) * (Z(mx) - Z(m - mx) * inv_n1)
    if n >= 3 and 2 * mx <= m:
        ans += ways(n - 2, m - 2 * mx, mx) * (Z(mx) - Z(m - 2 * mx) * inv_n2) * Z(n - 2)
print(ans)
