let (n, m, q) = read<(1..=2e5, 1..=2e5, 1..=2e5)>()
domain V = 1..=n
domain Street = 1..=m
let street = read<[Street](V, V)>()
let odd = street.ring.minimal(
    undoing(ParityDsu<V>(), add: (mut d, (u, v)) => d.unite(u, v)),
    ok: u => not u.state.consistent,
)
for (l, r) in read<[q](Street, Street)>():
    let open = m - (l..=r).len
    print(yes(odd[street.ring.prev(l)].map(w => w.len <= open) ?? false))
