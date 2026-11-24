// Source: IOI 2025 P3 (World Map), https://dmoj.ca/problem/ioi25p3
// limits: 1000 ms, 1024 MB
// Problem: n <= 40 countries and m distinct adjacent pairs of a connected graph; return a
//   K x K grid of countries, K <= 240, using every country, where two side-adjacent cells
//   of different countries are a listed pair and every listed pair touches somewhere. Full
//   score needs K <= 2n. Input and output as the IOI sample grader gives them: T <= 50
//   scenarios, each printed as K, the K row lengths, a blank line and the rows.
// Solution: an Euler tour of a DFS tree has 2n - 1 visits, consecutive ones joined by a tree
//   edge; paint one anti-diagonal of the 2n x 2n grid per visit. Every non-tree edge joins a
//   vertex to an ancestor, so it is held by whichever end has a visit nearer the tour's middle
//   (where diagonals are longest): that visit becomes three diagonals v | held | v, and with
//   n such triples the 4n - 1 diagonals fill the grid exactly. O(n^2) per scenario.
// original: corpus/problems/ioi25_worldmap/orig.cpp (the task archive's model solution plus
//   the sample grader)
// The judge form is dream.cp's `@grader fn create_map`, linked with dream.tests/grader.cpp;
// the port reads the sample grader's stdin so `cpl stress` and the --max gate run it alone.
// Bounds: t: 1..=50; n: 1..=40; m: 0..=780; ab: [m](a: 1..=n, b: 1..=n)
fn create_map<V, E>(pairs: [E](u: V, v: V)) -> [V.count * 2][V.count * 2]V:
    let n = V.count
    let g = Graph.undirected(pairs)
    let (tree, tree_edge) = g.dfs_tree(root: V.first!)
    let tour = tree.euler_tour().collect()
    let room = (i: Int) => min(i, 2 * n - 2 - i)  // how far visit i is from either end
    var best: [V]Int = 0                          // the visit of each vertex with the most room
    for (i, v) in tour.items:
        if room(i) > room(best[v]): best[v] = i
    var held: [V]List<V> = []
    for e in g.edges().filter(e => not tree_edge[e.e]):
        if room(best[e.u]) >= room(best[e.v]): held[e.u].push(e.v) else: held[e.v].push(e.u)

    var diags: List<List<V>> = []
    for (i, v) in tour.items:
        diags.push([v])
        if i == best[v]: diags.extend([held[v] + [v], [v]])
    // Cell (x, y) is on diagonal x + y, which enters the grid at row max(0, x + y - (2n - 1));
    // its place along the diagonal counts from there, and a short diagonal repeats its last colour.
    [2 * n] x => [2 * n] y =>
        let d = diags[x + y]
        d.get(x - max(0, x + y - (2 * n - 1))) ?? d.last!

let t = read<1..=50>()
for s in 0..t:
    let (n, m) = read<(1..=40, 0..=780)>()
    domain V = 1..=n
    let ab = read<[m](a: 1..=n, b: 1..=n)>()
    domain E = 0..m
    let map = create_map([E] i => (u: V(ab[i.ord].a), v: V(ab[i.ord].b)))
    if s > 0: print()
    print(2 * n)
    print([2 * n] _ => 2 * n)
    print()
    for row in map: print(row)
