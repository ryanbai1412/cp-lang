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

type Adj = [V]List<(to: V, w: Int)>

fn link(mut g: Adj, r: Road):
    g[r.u].push((to: r.v, w: r.len))
    g[r.v].push((to: r.u, w: r.len))

// Shortest roads first, cheapest among ties; keep a road only if the kept
// network doesn't already connect its ends within its length.
var g: Adj = []
var total = 0
for r in roads.sorted(by: r => (r.len, r.cost)):
    if (dijkstra(g, from: [(r.u, 0)])[r.v] ?? inf) > r.len:
        total += r.cost
        link(mut g, r)
print(total)
