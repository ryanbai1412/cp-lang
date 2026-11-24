# apio23p1 friction

## `Graph.only(...)` is not a `Next`, so the stop vertex needs a `[V]List` copy

- **Where:** `corpus/problems/apio23p1/spec.cp` wants
  `Graph.undirected(roads, keep: ...).only(from: v => v != goal)` (arriving at H ends the
  trip), then `dijkstra` over it. `Only<_, Graph<_, _>>` is E0606 "does not implement
  `WeightedNext`"; the same `.only` over a `[V]List` works, so the port builds one.
- **Repro:** `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2)]; print(reachable(Graph.directed(es).only(from: v => v != 1), from: [V(0)]).collect())`
  is E0606 (`Next` required by `reachable`); `let g: [V]List<V> = [[1], [2], []]` with the same
  call prints `0 1`. The `Only` doc says it "implements every arc capability `G` has".
- **Goal it hurt:** express (one fact, "nobody leaves H", becomes a hand-built adjacency).
- **Routed:** stdlib worklog.

## `opt.map(...) ?? []` types `[]` as `List<Int>` inside `flat_map`

- **Repro:** `let o: ?Int = Some(3); print((0..3).flat_map(i => o.map(d => [(i, d)]) ?? []).collect().len)`
  is E0302 "`flat_map` ... expected `Int`, found `(Int, Int)`": the empty list literal on the
  right of `??` does not take its element type from the left.
- **Goal it hurt:** express; the spec writes the halving layer as a loop pushing to `starts`.
- **Routed:** compiler worklog.

## A `@grader fn` has no local stub

- **Where:** spec R10.1.3 says local runs use a generated stub, but `cpl run` on a program whose
  only code is `@grader fn solve(...)` reads nothing and prints nothing, so `cpl-conform corpus`
  cannot judge it. The spec reads the sample grader's stdin format instead.
- **Routed:** tools worklog.

## Dream

```cp
// APIO 2023 "Cyberland": reach H from 0; some cities reset the elapsed time to
// 0, some halve it (at most k uses, and more than ~70 never helps). Arriving at
// H ends the trip. Layered Dijkstra: each layer is one more halving.
enum Power = Reset | Plain | Halve

@grader
fn solve(n: Int, m: Int, k: Int, h: Int, x: List<Int>, y: List<Int>, c: List<Int>, arr: List<Int>) -> F64:
    domain V = [n]
    let power = [V] v => Power.at(arr[v])!
    let goal = V(h)
    let roads = Graph<V>.undirected([x.keys] e => (u: V(x[e]), v: V(y[e]), w: F64(c[e])))
    let trip = roads.only(from: u => u != goal)
    let seen = reachable(trip, from: [V(0)])
    if not seen[goal]:
        return -1
    let starts = chain([V(0)], V.filter(v => seen[v] and power[v] == Reset))
    var dist = dijkstra(trip, from: starts.map(v => (v, 0.0))).dist
    for _ in 0..min(k, 70):
        let halved = V.filter(v => power[v] == Halve).flat_map(v =>
            dist[v].map(d => trip[v].map(a => (a.to, d / 2 + a.w))) ?? [])
        dist = dijkstra(trip, from: chain(dist.items.filter_map((v, d) => d.map(d => (v, d))), halved)).dist
    dist[goal]!
```
