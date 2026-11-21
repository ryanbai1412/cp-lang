// The gcd of all pairwise max flows is the gcd of the Gomory-Hu tree's weights.
// If it is not 1, one color. Otherwise two colors always suffice (Gallai): split V
// so each side's odd-weight edges give every vertex even degree; then every cut
// inside a side has even weight, so 2 divides all its max flows.
use std.flow.gomory_hu

@!multitest
let n: 1..=50 = read()
let m: 0..=1225 = read()
domain V = 1..=n
let edges = read<[m](u: V, v: V, w: 1..=1_000_000)>()

fn answer(colors: List<Bitset<V>>):
    print(colors.len)
    for c in colors:
        print(c.len)
        print(c)

var g = 0
for e in gomory_hu(edges): g = gcd(g, e.w)

if g != 1:
    answer([V.all])
else:
    var odd = adjacency(edges.filter(e => e.w % 2 == 1), into: Bitset)

    fn complement_within(s: Bitset<V>):
        for u in s: odd[u] ^= s.without(u)

    // Gallai's induction: remove an odd-degree v after complementing its neighborhood;
    // on the way back v joins the side where its degree is even.
    fn even_split(alive: Bitset<V>) -> Bitset<V>:
        let v = V.first(v => v in alive and (odd[v] & alive).len % 2 == 1) ?? return Bitset<V>.empty
        let rest = alive.without(v)
        let nbr = odd[v] & rest
        complement_within(nbr)
        var side = even_split(rest)
        complement_within(nbr)
        if (nbr & side).len % 2 == 0: side.add(v)
        side

    let side = even_split(V.all)
    answer([V.all - side, side])
