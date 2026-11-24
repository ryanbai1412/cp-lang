// Source: AtCoder ABC478 E (Inequality Sequence Construction),
//   https://atcoder.jp/contests/abc478/tasks/abc478_e
// Problem: find A of length n with values in 1..=n meeting q conditions A[u] <= A[v] (t = 0) or
//   A[u] < A[v] (t = 1); print Yes and A, or No. n, q <= 2e5.
// Solution: conditions are arcs u -> v of weight t. A strict arc inside a strongly connected
//   component is a contradiction; otherwise A is the longest path (from 1) in the condensation.
//   `scc` numbers components in topological order, so relaxing the arcs sorted by source component
//   sets a[to] max= a[from] + t in one pass, O(n + q log q); values stay <= the component count.

let (n, q): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let conds: [q](t: 0..=1, u: V, v: V) = read()
let (C, comp) = scc(Graph.directed([(u, v) for (_, u, v) in conds]))

if any(t == 1 and comp[u] == comp[v] for (t, u, v) in conds):
    print("No")
else:
    // components come in topological order, so relaxing arcs by source component settles each
    // component before it is read
    var a: [C]1..=n = 1
    for (t, u, v) in conds.sorted(by: c => comp[c.u]) if comp[u] != comp[v]:
        a[comp[v]] max= a[comp[u]] + t
    print("Yes")
    print([a[comp[v]] for v in V])
