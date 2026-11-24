// Tuned: every root's detective count comes from one rerooting pass, O(n), instead of
// relabelling the tree from each root; the plan keeps its steps unformatted until printed.
//
// Source: NOI 2007 P6, https://dmoj.ca/problem/noi07p6
// limits: 600 ms, 256 MB
// Problem: a tree on 1 <= N <= 1000 cities hides a thief who moves arbitrarily fast; a
//   detective in his city or on his road catches him. Steps: L x lands a detective at x, B x
//   recalls one, M x y moves one along an edge. Print S, the fewest detectives that surely
//   catch him, the step count T <= 20000 and the plan.
// Solution: rooted at r, a subtree whose child subtrees need a >= b >= ... detectives needs
//   max(a, b + 1) (a leaf needs 1): clear the lighter children one by one while a guard holds
//   the vertex, then walk into the heaviest one. Try every root, O(n^2 log n), and run the
//   plan from the best one: land on the root, clear lighter children and recall their
//   detectives, then step into the heaviest.
// original: corpus/submissions/3756630.cpp
// Bounds: n: 1..=1000; roads.u, roads.v: 1..=n
let n: 1..=1000 = read()
domain City = 1..=n
domain Road = 1..n
let roads: [Road](u: City, v: City) = read()
let g = Graph.undirected(roads, a => a.to)

// every root's count at once, rerooting: a vertex whose branches need a >= b (0 if missing)
// needs cover(a, b); down[v] is v's branch seen from its parent, up[v] the parent's side seen
// from v
fn cover(a: Int, b: Int) -> Int = if a == 0: 1 else: max(a, b + 1)
var down: [City]Int = 0
var up: [City]Int = 0
var whole: [City]Int = 0
fn go_down(u: City, p: ?City):
    var (a, b) = (0, 0)
    for v in g[u]:
        if Some(v) != p:
            go_down(v, Some(u))
            if down[v] > a: (a, b) = (down[v], a)
            elif down[v] > b: b = down[v]
    down[u] = cover(a, b)
fn go_up(u: City, p: ?City):
    // the three largest branches of u, and the neighbours the first two lie behind
    var (a, b, c) = (0, 0, 0)
    var (by_a, by_b): (?City, ?City) = (None, None)
    for v in g[u]:
        let x = if Some(v) == p: up[u] else: down[v]
        if x > a: (a, b, c, by_a, by_b) = (x, a, b, Some(v), by_a)
        elif x > b: (b, c, by_b) = (x, b, Some(v))
        elif x > c: c = x
    whole[u] = cover(a, b)
    for v in g[u]:
        if Some(v) != p:
            up[v] = if by_a == Some(v): cover(b, c) elif by_b == Some(v): cover(a, c) else: cover(a, b)
            go_up(v, Some(u))
go_down(City.first!, None)
go_up(City.first!, None)
let root = City.min(by: r => whole[r])!
go_down(root, None)  // down[v] is now what v's subtree needs, rooted at root
print(down[root])

var plan: List<(step: Str, at: City, to: ?City)> = []  // the steps, printed once T is known
fn land(u: City): plan.push((step: "L", at: u, to: None))

// clear the subtree of u; `guarded`: a detective already stands on u and stays in the
// subtree, else one detective comes back up to p
fn sweep(u: City, p: ?City, guarded: Bool):
    var heavy: ?City = None
    for v in g[u]:
        if Some(v) != p and (heavy == None or down[v] > down[heavy!]):
            heavy = Some(v)
    let Some(h) = heavy else:
        if not guarded: land(u)
        if not guarded: plan.push((step: "M", at: u, to: p))
        return
    if not guarded: sweep(h, Some(u), false)
    for v in g[u]:
        if Some(v) != p and v != h:
            sweep(v, Some(u), false)
            plan.push((step: "B", at: u, to: None))
    if guarded:
        plan.push((step: "M", at: u, to: Some(h)))
        sweep(h, Some(u), true)
    else:
        plan.push((step: "M", at: u, to: p))

land(root)
sweep(root, None, true)
print(plan.len)
for s in plan:
    match s.to:
        Some(v) => print(s.step, s.at, v)
        None => print(s.step, s.at)
