// Source: CCC 2023 S4 (Minimum Cost Roads), https://dmoj.ca/problem/ccc23s4
// Problem: N, M <= 2000; road i joins u_i, v_i with length l_i <= 1e9 and cost c_i <= 1e9.
//   Keep a subset of roads of least total cost such that every pair of intersections keeps its
//   shortest distance. Print that cost.
// Solution: take roads by (length, cost); keep a road iff the kept roads do not already join its
//   ends within its length (a Dijkstra from one end over the kept graph). Shorter roads never
//   need longer ones, so this is exact. O(M (N + M) log N).
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
