// btoi20p3 Joker: does closing streets l..=r still leave an odd cycle among the rest?
// The open streets are the wrapping window of the street ring from r.next round to l.prev, and
// "holds an odd cycle" survives growing the window. So one two-pointer pass round the ring finds,
// for each end, the shortest odd window ending there; a query asks if it fits in m - len(l..=r).
// A parity DSU only undoes its newest union, and `undoing` makes it a front-popping `Track`.
let (n, m, q) = read<(1..=2e5, 1..=2e5, 1..=2e5)>()
domain V = 1..=n
domain Street = 1..=m
let street = read<[Street](V, V)>()
let odd = street.ring.minimal(
    undoing(ParityDsu<V>(), add: (mut d, (u, v)) => d.unite(u, v)),
    ok: u => not u.state.consistent,
)
for closed in read<[q]Range<Street>>():
    let shortest = odd[street.ring.prev(closed.l)]
    print(yes(shortest.is_some_and(w => w.len <= m - closed.len)))
