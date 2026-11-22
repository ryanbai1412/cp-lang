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
