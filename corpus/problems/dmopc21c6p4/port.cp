// Source: DMOPC 2021 Contest 6 P4 (Colourful Paths), https://dmoj.ca/problem/dmopc21c6p4
// Problem: a connected simple graph, n, m <= 2e5, and distinct houses A, B, C, D. Colour the
//   roads so that some A-B path is one colour and every C-D path has two colours, using the
//   fewest colours and then the shortest one-colour A-B path. Print -1 or K and the colours.
// Solution: K = 2: roads at C get colour 1, roads at D colour 2 (a C-D road makes it -1). An A-B
//   path stays one colour iff it does not pass both C and D, so BFS over (city, which of C and D
//   were met) finds the shortest such path, and every other road takes the colour that path
//   needs (2 if it met D, else 1). O(n + m).
// original: corpus/submissions/4342347.cpp
// No top-level early exit on the prototype, so the program lives in a function.

let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
domain City = 1..=n
let roads: [m](u: City, v: City) = read()
let a: City = read()
let b: City = read()
let c: City = read()
let d: City = read()
domain State = (at: City, met_c: Bool, met_d: Bool)

let near = Graph.undirected(roads)

fn enter(v: City, met_c: Bool, met_d: Bool) -> State = State((v, met_c or v == c, met_d or v == d))
fn moves(s: State) -> yields State:
    for v in near.next(s.at): yield enter(v, s.met_c, s.met_d)
let dist = bfs(moves, from: [enter(a, false, false)])

// the shortest A-B path that does not meet both C and D, and whether it met D
var best: ?(len: Int, met_d: Bool) = None
for met_d in Bool:
    for met_c in Bool if not (met_c and met_d):
        if let x = dist[State((b, met_c, met_d))]:
            if best == None or x < best!.len: best = (x, met_d)

fn touches(r: (u: City, v: City), x: City) -> Bool = r.u == x or r.v == x
if any(touches(r, c) and touches(r, d) for r in roads):
    print(-1)
elif let found = best:
    let rest = if found.met_d: 2 else: 1
    print(2)
    for r in roads: print(if touches(r, c): 1 elif touches(r, d): 2 else: rest)
else:
    print(-1)
