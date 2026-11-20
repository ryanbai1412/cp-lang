// NOI 2007 Social Network: a connected undirected weighted graph on n <= 100
// people. The importance of v is the sum, over ordered pairs (s, t) with
// s, t, v distinct, of the fraction of shortest s-t paths that pass
// through v. Print every importance to 3 decimals.
//
// Floyd-Warshall carrying the number of shortest paths next to the length;
// v is on a shortest s-t path when the lengths through it add up, and then
// it is on ways[s][v] * ways[v][t] of them.
// original: corpus/submissions/5032737.cpp

// Three decimals, as the judge prints them. The interpreter has no `{x:.3f}`.
fn fixed3(x: Float) -> Str:
    let units = Int(x * 1000 + 0.5)
    let frac = units % 1000
    "{units / 1000}.{frac / 100}{frac / 10 % 10}{frac % 10}"

let n: Int = read()
let m: Int = read()
domain P = 0..n
var dist: [P, P]?Int = none
var ways: [P, P]Int = 0
for _ in 0..m:
    let a: Int = read()
    let b: Int = read()
    let c: Int = read()
    let (u, v) = (P(a - 1), P(b - 1))
    dist[u, v] = c
    dist[v, u] = c
    ways[u, v] = 1
    ways[v, u] = 1

for k in P:
    for s in P:
        let a = dist[s, k] else: continue
        for t in P:
            let b = dist[k, t] else: continue
            if s == t: continue
            if dist[s, t] == none or a + b < dist[s, t]!:
                dist[s, t] = a + b
                ways[s, t] = ways[s, k] * ways[k, t]
            elif a + b == dist[s, t]!:
                ways[s, t] += ways[s, k] * ways[k, t]

for v in P:
    var importance = 0.0
    for s in P:
        for t in P:
            if s == t or s == v or t == v: continue
            if dist[s, v]! + dist[v, t]! == dist[s, t]!:
                importance += Float(ways[s, v] * ways[v, t]) / Float(ways[s, t])
    print(fixed3(importance))
