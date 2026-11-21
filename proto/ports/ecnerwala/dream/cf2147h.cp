// The gcd of all pairwise max flows is the gcd of the Gomory–Hu tree's weights.
// If it is not 1, one color. Otherwise two colors always suffice (Gallai): split V
// so each side's odd-weight edges give every vertex even degree; then every cut
// inside a side has even weight, so 2 divides all its max flows.
use std.flow.{gomory_hu}
@!multitest
domain V = read(1..=50)
let m: 0..=choose2(V.count) = read()
let edges = read<[m](u: V, v: V, w: 1..=1e6)>()

fn answer(colors: List<Bitset<V>>):
    print(colors.len)
    for c in colors:
        print(c.len)
        print(c)

if gcd(gomory_hu(edges).map(e => e.w)) != 1: return answer([V.all])

var odd = adjacency(edges.filter(e => e.w % 2 == 1), into: Bitset)

fn complement_within(s: Bitset<V>):          // toggle every edge inside s
    for u in s: odd[u] ^= s.without(u)

// Gallai's induction: an odd-degree v is removed after complementing its
// neighbourhood; on the way back v joins the side where its degree is even.
fn even_split(alive: Bitset<V>) -> Bitset<V>:
    let v = alive.first(v => (odd[v] & alive).len % 2 == 1) ?? return Bitset.empty
    let rest = alive.without(v)
    let nbr = odd[v] & rest
    complement_within(nbr)
    var side = even_split(rest)
    complement_within(nbr)
    if (nbr & side).len % 2 == 0: side.add(v)
    return side

let side = even_split(V.all)
answer([V.all - side, side])
