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

var fg = FlowGraph<V>()
for e in edges: fg.add_both(e.u, e.v, e.w)
var par: [V]V = V.first
var g = 0
for i in V if i != V.first:
    fg.reset()
    g = gcd(g, fg.max_flow(i, par[i]))
    let cut = fg.min_cut(i)
    for j in V.after(i) if par[j] == par[i] and cut[j]: par[j] = i

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
    answer([all ^ side, side])
