# 28. Graphs and trees

**Layer:** library. Status: [proposed].

Graphs are CSR arrays over a domain; `bfs` is generic over any domain and a `next:` function, so grids,
product states and implicit graphs use the same search. Nothing here needs compiler support; the fresh
domains `components` returns use the existential return of chapter 12.2.

## 28.1 `Graph<V>`

- `Graph<V>`: CSR adjacency; `.adj(v)` yields `V`, `.out(v)` yields `(V, W)` when weighted [r16 R12], `.edges` view, `.undirected(from:, to:, weight:)`; `Edge<V, W>`; `g.components() -> (domain C, Partition)` [p1 F4]

## 28.2 Searches and builders [p3]

  - `bfs(from: roots, next: f) -> Search<D>` over any domain [r16 R10–R11]: `.dist: [D]?(0..D.count)`, `.parent`,
    `.order`, `.path(to:)`, `.tree()`, `.root` with `roots: true`. Sources dequeue in `from:` order and each
    level keeps its parents' order. It replaces `g.bfs_dist`, `g.bfs_order` and `g.bfs_tree`; on a graph,
    `next: g.adj`.
  - Grids: `cell.on_border`; stdlib `Dir4 = ring(labels(N, E, S, W))` with `cell.step(d) -> ?Cell` and
    `a.dir_to(b) -> ?Dir4`.
  - `Tree(edges).rooted(at: roots)` (forests)
  - `g.cycle() -> ?List<V>`
  - `t.euler()`, with `pos: [V]Order` and `span: [V]Range<Order>`
  - one constructor family: `Graph<V>.undirected(…)` / `.directed(…)`, each taking `edges:`, `from:`/`to:`,
    or `adj:`; children from a parent array are `I.group(by: parent)`
  - `g.topo_order() -> ?[V.count]V` (`none` on a cycle)
