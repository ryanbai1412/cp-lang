// Source: DMOPC '20 Contest 6 P3 (Bread Merchant), https://dmoj.ca/problem/dmopc20c6p3
// Problem: directed graph, N, M <= 5e5 (self-loops allowed); some cities have police
//   (p_i = 1). The robber must leave any city with an out-route: police pick the route,
//   elsewhere the robber picks. He is caught on reaching a city with no out-route. For each
//   start city print 1 if optimal police catch him, else 0 (running forever is not caught).
// Solution: the police win exactly on the attractor of the dead ends. Walk it backwards from
//   the dead ends: a police city joins once one of its routes leads into it, a robber city
//   once all of them do (`free` counts routes not yet known to lead in). Every other city
//   lets the robber stay out forever. O(N + M).
// original: corpus/submissions/3917634.cpp
// Bounds: n, m: 1..=5e5; police: 0..=1; routes.u, routes.v: 1..=n
let n: 1..=5e5 = read()
let m: 1..=5e5 = read()
domain City = 1..=n
domain Route = 0..m
let police: [City]0..=1 = read()
let routes: [Route](u: City, v: City) = read()

let g = Graph.directed(routes)
let back = g.transpose()
var free = [City] u => g.deg(u)
var caught = [City] u => free[u] == 0
var fresh = [u for u in City if caught[u]]
while let u = fresh.pop():
    for arc in back.out(u):
        let p = arc.to
        if not caught[p]:
            free[p] -= 1
            if police[p] == 1 or free[p] == 0:
                caught[p] = true
                fresh.push(p)
print([Int(caught[u]) for u in City])
