// ucrpc21h: four couriers share round trips from the depot, each courier's day at most 600. A
// round trip to v costs 2 dist(v), so split the shortest distances into four loads of at most 300:
// a subset-sum over three loads, the third load a bitset, the fourth taking the rest.
const limit = 300
let (n, m) = read<(1..=100, 0..=10_000)>()
domain V = [n]
let roads = read<[m](u: V, v: V, w: 1..=1_000_000_000)>()
let paths = dijkstra(Graph.undirected(roads), from: [(V.first!, 0)])
fn busiest() -> ?Int:
    if roads.any(r => r.w > limit) or V.any(v => paths.dist[v] == None):
        return None
    let d = [V] v => paths.dist[v]!
    let total = d.sum()
    if d.max()! > limit or total > 4 * limit:
        return None
    domain Load = 0..=min(limit, total)
    var can: [Load, Load]Bitset<Load> = Bitset<Load>.new()
    can[0, 0] = Bitset<Load>.single(Load(0))
    for v in V:
        if d[v] > 0:
            for (a, b) in (Load, Load).rev:
                var next = can[a, b].shifted(d[v])
                if let a2 = Load.at(a - d[v]): next.union(can[a2, b])
                if let b2 = Load.at(b - d[v]): next.union(can[a, b2])
                can[a, b].union(next)
    (Load, Load).flat_map(((a, b)) => can[a, b].each().map(c => max(Int(a), Int(b), Int(c), total - Int(a) - Int(b) - Int(c)))).min()
if let day = busiest() and day <= limit:
    print(2 * day)
else:
    print("Impossible!")
