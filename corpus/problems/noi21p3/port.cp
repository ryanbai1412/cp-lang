// Source: NOI 2021 P3 (Celebration), https://dmoj.ca/problem/noi21p3
// limits: 2000 ms, 1024 MB
// Problem: a weakly connected digraph, n, q <= 3e5, m <= 6e5, where x => z and y => z imply
//   x => y or y => x. Each query gives s, t and k <= 2 extra edges usable only by it; print
//   how many cities lie on some s-to-t walk (0 if t is unreachable).
// Solution: condense SCCs, weighted by size. The property makes each component's
//   predecessors a chain, so the condensed DAG's transitive reduction is an out-tree: each
//   component hangs under its last predecessor in a topological order. Per query, take the
//   virtual tree of the <= 6 key components plus the extra edges, take what s reaches and
//   what reaches t in it, and count each component (or inner path of a tree edge) both reach.
//   O(n + m) setup, O(k log n) per query.
// Bounds: n: 1..=3e5; m: n - 1..=6e5; q: 1..=3e5; k: 0..=2; edges.u, edges.v: 1..=n; s, t: 1..=n; extra.u, extra.v: 1..=n

let n: 1..=3e5 = read()
let (m, q, k): (n - 1..=6e5, 1..=3e5, 0..=2) = read()  // weakly connected: m >= n - 1
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let g = Graph<V, E>.directed(edges)

let (C, comp) = scc(g)
var size: [C]Int = 0
for u in V: size[comp[u]] += 1

// The predecessors of a component form a chain, so its last one in topological order is its
// parent in the transitive reduction, which is a tree.
let dag = condense(g, comp)
let order = topo(dag)!
var pos: [C]Int = 0
for (i, c) in order.items: pos[c] = i
var up: [C]?C = None
for c in C:
    for a in dag[c]:
        if up[a.to] == None or pos[c] > pos[up[a.to]!]: up[a.to] = c
var kids: [C]List<C> = []
for c in C:
    if let p = up[c]: kids[p].push(c)
let tree = Rooted(u => [(to: v, w: size[v]) for v in kids[u]], order[0])

repeat q:
    let (s, t): (V, V) = read()
    let extra: [k](u: V, v: V) = read()
    let vt = tree.virtual([comp[s], comp[t]] + [comp[e.u] for e in extra] + [comp[e.v] for e in extra])
    fn node(u: V) -> vt.K = vt.K.find(x => vt.at[x] == comp[u])!
    var ahead: [vt.K]List<vt.K> = []
    var behind: [vt.K]List<vt.K> = []
    for x in vt.K:
        if let p = vt.parent[x]:
            ahead[p].push(x)
            behind[x].push(p)
    for e in extra:
        ahead[node(e.u)].push(node(e.v))
        behind[node(e.v)].push(node(e.u))
    var from_s: [vt.K]Bool = false
    var to_t: [vt.K]Bool = false
    for x in reachable(x => ahead[x], [node(s)]): from_s[x] = true
    for x in reachable(x => behind[x], [node(t)]): to_t[x] = true

    var cities = 0
    for x in vt.K:
        if from_s[x] and to_t[x]: cities += size[vt.at[x]]
        // the inner components of the tree edge into x
        if let p = vt.parent[x], from_s[p] and to_t[x]: cities += vt.w[x] - size[vt.at[x]]
    print(cities)
