// UCRPC 2021 H, four couriers splitting round trips within 600
// original: corpus/submissions/3950286.cpp
const LIMIT = 300
let n: 1..=100 = read()
let m: 0..=1e4 = read()
domain V = 0..n
let es: [0..m](u: V, v: V, w: Int) = read()

let dist = dijkstra(Graph.undirected(es), from: V.first)
let trips = [d! for d in dist]             // the statement promises a connected graph
let total = trips.sum()

if any(e.w > LIMIT for e in es) or trips.max() > LIMIT or total > 4 * LIMIT:
    print("Impossible!")
    return

// split[a, b, c]: three couriers can carry exactly a, b and c; the fourth takes the rest
domain Load = 0..=min(LIMIT, total)
var split: [Load, Load, Load]Bool = false
split[0, 0, 0] = true
for x in trips:
    for (a, b, c) in (Load * Load * Load).rev:
        split[a, b, c] or= (a >= x and split[a - x, b, c]) or (b >= x and split[a, b - x, c]) or (c >= x and split[a, b, c - x])

let best = min(max(a, b, c, total - a - b - c) for (a, b, c) in Load * Load * Load if split[a, b, c])
if best <= LIMIT: print(2 * best) else: print("Impossible!")
