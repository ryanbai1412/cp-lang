// BOI 2020 "Joker": closing streets l..=r leaves the rest, a wrapping piece of the street ring.
// It has an odd cycle iff it contains the shortest odd-cycle piece ending just before l.
let (n, m, q): (Int, Int, Int) = read()
domain V = 1..=n
domain Street = 1..=m
let street: [Street](u: V, v: V) = read()
let odd = street.ring.minimal(
    state: undoing(ParityDsu<V>(), add: (mut d, s) => { let _ = d.unite(s.u, s.v) }),
    ok: d => not d.state.consistent(),
)
for closed in read<[q](l: Street, r: Street)>():
    let shortest = odd[street.ring.prev(closed.l)]
    print(yes(shortest.is_some_and(w => w.len <= m - (closed.r.ord - closed.l.ord + 1))))
