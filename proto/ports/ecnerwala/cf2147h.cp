// The gcd of all pairwise max flows is the gcd of the Gomory-Hu tree's weights.
// If it is not 1, one color. Otherwise two colors always suffice (Gallai): split V
// so each side's odd-weight edges give every vertex even degree; then every cut
// inside a side has even weight, so 2 divides all its max flows.
use std.flow.{gomory_hu}
@!multitest
let n: 1..=50 = read()
let m: 0..=1225 = read()
domain V = 1..=n
let edges: [0..m](u: V, v: V, w: 1..=1_000_000) = read()

fn answer(colors: List<Bitset<V>>):
    print(colors.len)
    for c in colors:
        print(c.len)
        print([v for v in c.each()])

var g = 0
for e in gomory_hu(edges): g = gcd(g, e.w)

let all = Bitset<V>.full()
if g != 1:
    answer([all])
else:
    var odd: [V]Bitset<V> = [V] _ => Bitset<V>.new()
    for e in edges if e.w % 2 == 1:
        odd[e.u].toggle(e.v)
        odd[e.v].toggle(e.u)

    fn complement_within(s: Bitset<V>):
        for u in s.each():
            odd[u] ^= s
            odd[u].remove(u)

    // Gallai's induction: remove an odd-degree v after complementing its neighborhood;
    // on the way back v joins the side where its degree is even.
    fn even_split(alive: Bitset<V>) -> Bitset<V>:
        let v = first(
            v for v in alive.each() if (odd[v] & alive).len % 2 == 1
        ) ?? return Bitset<V>.new()
        var rest = alive
        rest.remove(v)
        let nbr = odd[v] & rest
        complement_within(nbr)
        var side = even_split(rest)
        complement_within(nbr)
        if (nbr & side).len % 2 == 0: side.add(v)
        side

    let side = even_split(all)
    answer([all - side, side])
