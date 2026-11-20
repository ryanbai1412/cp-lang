// cp: 0.1 8ef64b9+dirty
let n: 1..=2000 = read()
let m: 1..=2000 = read()
domain V = 1..=n
domain E = 0..m

struct Road:
    u, v: V
    len: 1..=1e9
    cost: 1..=1e9

let roads: [E]Road = read()

type Adj = [V]List<(V, Int)>

fn link(mut g: Adj, r: Road):
    g[r.u].push((r.v, r.len))
    g[r.v].push((r.u, r.len))

// Shortest roads first, cheapest among ties; keep a road only if the kept
// network doesn't already connect its ends within its length.
var g: Adj = []
var total = 0
for r in roads.sorted_by(|r| (r.len, r.cost)):
    if (dijkstra(g, r.u)[r.v] ?? inf) > r.len:
        total += r.cost
        link(mut g, r)
print(total)
