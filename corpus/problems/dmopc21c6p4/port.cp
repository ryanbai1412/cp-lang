// Source: DMOPC 2021 Contest 6 P4 (Colourful Paths), https://dmoj.ca/problem/dmopc21c6p4
// limits: 3000 ms, 1024 MB
// Problem: a connected simple graph, n, m <= 2e5, and distinct houses A, B, C, D. Colour the
//   roads so that some A-B path is one colour and every C-D path has two colours, using the
//   fewest colours and then the shortest one-colour A-B path. Print -1 or K and the colours.
// Solution: K = 2: roads at C get colour 1, roads at D colour 2 (a C-D road makes it -1). An A-B
//   path stays one colour iff it does not pass both C and D, so the shorter of a BFS that skips D
//   and one that skips C finds it, and every other road takes the colour that path needs (2 if
//   it met D, else 1). O(n + m).
// original: corpus/submissions/4342347.cpp
// No top-level early exit on the prototype, so the program lives in a function.
// Bounds: n, m: 1..=2e5; roads.u, roads.v, a, b, c, d: 1..=n

let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
domain City = 1..=n
let roads: [m](u: City, v: City) = read()
let a: City = read()
let b: City = read()
let c: City = read()
let d: City = read()
let near = Graph.undirected(roads)

// the shortest A-B path that does not meet both C and D: one that skips D, else one that skips C
let skip_d = bfs(near, from: [a], enter: (v, _) => v != d)[b]
let skip_c = bfs(near, from: [a], enter: (v, _) => v != c)[b]
let best: ?(len: Int, met_d: Bool) = match (skip_d, skip_c):
    (Some(x), Some(y)) => if x <= y: Some((x, false)) else: Some((y, true))
    (Some(x), None) => Some((x, false))
    (None, Some(y)) => Some((y, true))
    _ => None

fn touches(r: (u: City, v: City), x: City) -> Bool = r.u == x or r.v == x
if any(touches(r, c) and touches(r, d) for r in roads):
    print(-1)
elif let found = best:
    let rest = if found.met_d: 2 else: 1
    print(2)
    for r in roads: print(if touches(r, c): 1 elif touches(r, d): 2 else: rest)
else:
    print(-1)
