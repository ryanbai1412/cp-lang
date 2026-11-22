// ucrpc21h: four couriers share round trips from the depot, each courier's day at most 600. A
// round trip to v costs 2 dist(v), so split the shortest distances into four loads of at most 300:
// a 3-D subset-sum over three loads, the fourth taking the rest.
const limit = 300
let (n, m) = read<(1..=100, 0..=10_000)>()
domain V = [n]
let roads = read<[m](u: V, v: V, w: 1..=1_000_000_000)>()
let paths = dijkstra(Graph.undirected(roads), from: [(V.first!, 0)])
fn busiest() -> ?Int:
    if roads.any(r => r.w > limit) or V.any(v => paths.dist[v] == none):
        return none
    let d = [V] v => paths.dist[v]!
    let total = d.sum()
    if d.max()! > limit or total > 4 * limit:
        return none
    domain Load = 0..=min(limit, total)
    domain Split = (Load, Load, Load)
    var can: [Split]Bool = false
    can[0, 0, 0] = true
    for v in V:
        if d[v] > 0:
            for (a, b, c) in Split.rev:
                for from in [(a - d[v], b, c), (a, b - d[v], c), (a, b, c - d[v])]:
                    if let p = Split.at(from):
                        can[a, b, c] = can[a, b, c] or can[p]
    Split.filter(p => can[p]).map(((a, b, c)) => max(max(a, b), max(c, total - a - b - c))).min()
if let day = busiest() and day <= limit:
    print(2 * day)
else:
    print("Impossible!")
