# Median constraints: find a[1..n] in 1..=1e9 with median(a[i], a[j], a[k]) = x for each query, or
# say NO. For every (v, x) a query names, keep two facts: "a[v] < x" and "a[v] > x". At most one of
# a query's three values is below x and at most one above. Along one vertex's thresholds in
# increasing x, the facts are monotone: "a[v] > x" for one threshold means "a[v] >= next", and so on.
# That is 2-SAT.
domain V = [read Int]
let asks = read [read Int] (i: V, j: V, k: V, x: Int)
let pins = [V] v => asks.filter(q => v in [q.i, q.j, q.k]).map(.x).distinct().sorted()
var sat = TwoSat()
let below = [V] v => pins[v].keyed(by: x => x).map(_ => sat.var())
let above = [V] v => pins[v].keyed(by: x => x).map(_ => sat.var())
for q in asks:
    for (u, w) in [q.i, q.j, q.k].pairs():
        sat.at_most_one(below[u][q.x], below[w][q.x])
        sat.at_most_one(above[u][q.x], above[w][q.x])
for v in V:
    for x in pins[v]: sat.at_most_one(below[v][x], above[v][x])
    for (x, y) in pins[v].with_next():
        sat.exactly_one(above[v][x], below[v][y])
match sat.solve():
    None => print("NO")
    Some(truth) =>
        print("YES")
        print(V.map(v =>
            let lo = max(1, pins[v].filter(x => not truth[below[v][x]]).map(x => if truth[above[v][x]]: x + 1 else: x).max() ?? 1)
            lo))
