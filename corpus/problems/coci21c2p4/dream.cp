type Z = Mod<1e9 + 7>
let (n, l) = read<(1..=50, 1..=1e4)>()
let r = read<[n]1..=1e4>().sorted(by: desc)
let comb = Comb<Z>(l + n)
domain State = (comps: 0..=n, used: 0..l, ends: 0..=2)

@memo(array)
fn ways(i: 0..=n, st: State) -> Z:
    let (c, u, e) = st
    if i == n:
        return Z(c == 1 and u == l - 1 and e == 0)
    fn go(dc: Int, du: Int, de: Int) -> Z =
        State.at((c + dc, u + du, e + de)).map(t => ways(i + 1, t)) ?? 0
    let s = r[i]
    var total = (c - 1 + e) * go(1, 2 * s, 0) + e * go(1, s, -1)
    if c >= 1:
        total += (2 * c - 2 + e) * go(0, s, 0) + e * go(0, 0, -1) + (c - 1) * go(-1, 0, 0)
    total

print(if n == 1: l else: (0..l).sum(u => ways(0, (0, u, 2)) * comb.choose(u + n, n)))
