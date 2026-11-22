// UCRPC 2021 H: split the shortest distances from 0 among four walkers so the longest round trip
// is smallest (at most 600, else "Impossible!"). Edges over 300 already make it impossible.
// Reachable (a, b, c) loads for three walkers form a bitset DP; the fourth gets the rest.
let (n, m) = read (Int, Int)
domain V = [n]
let roads = read [m](u: V, v: V, w: Int)
if roads.any(r => r.w > 300): print("Impossible!"); return
let dist = dijkstra(Graph.undirected(roads), from: [V.first!]).dist.map(d => d!)
domain Load = 0..=300
var can: [Load, Load, Load]Bool = false
can[0, 0, 0] = true
for d in dist:
    can = [(a, b, c) in can.keys] => can[a, b, c]
        or Load.at(Int(a) - d).is_some_and(a2 => can[a2, b, c])
        or Load.at(Int(b) - d).is_some_and(b2 => can[a, b2, c])
        or Load.at(Int(c) - d).is_some_and(c2 => can[a, b, c2])
let total = dist.sum()
let best = can.items.filter_map(((a, b, c), ok) => if ok: Some(max(a, b, c, total - a - b - c)) else: None).min()!
print(if 2 * best <= 600: "{2 * best}" else: "Impossible!")
