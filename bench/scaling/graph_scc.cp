// fixed: k, max_w, d_max, moves
use std.mod.{Mod}
use std.graph.{Graph, Lit, TwoSat, Functional, IncrementalBridges, adjacency, arborescence, arc_len, bfs, bfs01, block_cut_tree, blocks, boruvka, bridges, complement, components, condense, count_paths, cut_vertices, dag_fold, dfs, dial, dijkstra, directed_of_pairs, dominators, euler_walk, find_cycle, fixed_cost_paths, k_shortest, kruskal, odd_cycle, peel, reachable, scc, scc_times, smallest_topo, spanning_forest, topo, triangles, two_color, two_edge_components, undirected_of_pairs}
// Sparse graphs on n vertices: `cyc` has arcs i -> i+1 and i -> i+3 (mod n, n even), so it is
// Eulerian, strongly connected and bipartite; `dag` has the same arcs without wrapping.
let n: Int = read()
domain V = 0..n
domain E = 0..2 * n
domain D = 0..2 * n - 4
let es: [E](u: V, v: V) = [E] e => (u: V(e.ord % n), v: V((e.ord % n + 1 + 2 * (e.ord / n)) % n))
let wes: [E](u: V, v: V, w: Int) = [E] e => (u: es[e].u, v: es[e].v, w: e.ord % 10 + 1)
let des: [D](u: V, v: V) = [D] e => if e.ord < n - 1: (u: V(e.ord), v: V(e.ord + 1)) else: (u: V(e.ord - n + 1), v: V(e.ord - n + 4))
let cyc = [V] v => [V((v.ord + 1) % n), V((v.ord + 3) % n)]
let und = [V] v => [V((v.ord + 1) % n), V((v.ord + 3) % n), V((v.ord + n - 1) % n), V((v.ord + n - 3) % n)]
let dag = [V] v => [V(u) for u in [v.ord + 1, v.ord + 3] if u < n]
let pairs = [(es[e].u, es[e].v) for e in E]
let arcs = [(u: es[e].u, v: es[e].v) for e in E]
let from_arcs = [(from: es[e].u, arc: es[e].v) for e in E]
var s = 0
for _ in 0..16:
    s += scc_times(es).count(t => t != None) // timed: graph.scc_times
for _ in 0..32:
    let (domain K, sc) = scc(cyc) // timed: graph.scc
let (domain K0, sc0) = scc(cyc)
for _ in 0..96:
    s += condense(cyc, sc0)[sc0[V(0)]].len // timed: graph.condense
print(s)
