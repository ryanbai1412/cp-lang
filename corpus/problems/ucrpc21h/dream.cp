// UCRPC 2021 H: split the shortest distances from 0 among four walkers so the longest round trip
// is smallest (at most 600, else "Impossible!"). Edges over 300 already make it impossible.
// Reachable (a, b, c) loads for three walkers form a DP with a bitset over c; the fourth gets the rest.
const limit = 300
let (n, m): (1..=100, 0..=10_000) = read()
domain V = [n]
let roads = read<[m](u: V, v: V, w: 1..=1_000_000_000)>()
let dist = dijkstra(Graph.undirected(roads), from: [(V.first!, 0)]).dist

fn busiest() -> ?Int:
    if roads.any(r => r.w > limit) or V.any(v => dist[v] == None): return None
    let d = [V] v => dist[v]!
    let total = d.sum()
    if d.max()! > limit or total > 4 * limit: return None
    domain Load = 0..=min(limit, total)
    var can: [Load, Load]Bitset<Load> = Bitset<Load>.new()
    can[0, 0] = Bitset<Load>.single(Load(0))
    for x in d if x > 0:
        for (a, b) in (Load, Load).rev:
            var next = can[a, b].shifted(x)
            if let a2 = Load.at(a - x): next.union(can[a2, b])
            if let b2 = Load.at(b - x): next.union(can[a, b2])
            can[a, b].union(next)
    (Load, Load).flat_map(((a, b)) => can[a, b].each().map(c => max(Int(a), Int(b), Int(c), total - Int(a) - Int(b) - Int(c)))).min()

if let day = busiest() and day <= limit: print(2 * day) else: print("Impossible!")
