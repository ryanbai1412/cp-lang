// Source: NOI 2007 P6, https://dmoj.ca/problem/noi07p6
// limits: 600 ms, 256 MB
// Problem: a tree on 1 <= N <= 1000 cities hides a thief who moves arbitrarily fast; a
//   detective in his city or on his road catches him. Steps: L x lands a detective at x, B x
//   recalls one, M x y moves one along an edge. Print S, the fewest detectives that surely
//   catch him, the step count T <= 20000 and the plan.
// Solution: rooted at r, a subtree whose child subtrees need a >= b >= ... detectives needs
//   max(a, b + 1) (a leaf needs 1): clear the lighter children one by one while a guard holds
//   the vertex, then walk into the heaviest one. Reroot once to find each root's need: up[v]
//   is what the tree beyond v's parent needs, from the parent's other branches. O(sum deg^2).
//   Run the plan from the best root: land on it, clear lighter children and recall their
//   detectives, then step into the heaviest.
// original: corpus/submissions/3756630.cpp
// Bounds: n: 1..=1000; roads.u, roads.v: 1..=n
let n: 1..=1000 = read()
domain City = 1..=n
domain Road = 1..n
let roads: [Road](u: City, v: City) = read()
let g = Graph.undirected(roads, a => a.to)

fn children(u: City, p: ?City) -> List<City> = [v for v in g[u] if Some(v) != p]

// detectives that clear a part whose branches need `xs`
fn cover(xs: List<Int>) -> Int:
    let top = xs.max_k(2).sorted(by: x => desc(x))
    if top.len == 0: 1 elif top.len == 1: top[0] else: max(top[0], top[1] + 1)

var down: [City]Int = 0  // the subtree of u entered from its parent
var up: [City]Int = 0  // everything beyond u's parent, entered from u
var whole: [City]Int = 0  // the tree rooted at u
fn go_down(u: City, p: ?City):
    for v in children(u, p): go_down(v, Some(u))
    down[u] = cover([down[v] for v in children(u, p)])
fn branch(u: City, p: ?City, v: City) -> Int = if Some(v) == p: up[u] else: down[v]
fn go_up(u: City, p: ?City):
    whole[u] = cover([branch(u, p, v) for v in g[u]])
    for v in children(u, p):
        up[v] = cover([branch(u, p, w) for w in g[u] if w != v])
        go_up(v, Some(u))

go_down(City.first!, None)
go_up(City.first!, None)
let root = City.min(by: r => whole[r])!
go_down(root, None)
print(down[root])

var plan: List<Str> = []
fn land(u: City): plan.push("L {u}")

// clear the subtree of u; `guarded`: a detective already stands on u and stays in the
// subtree, else one detective comes back up to p
fn sweep(u: City, p: ?City, guarded: Bool):
    let cs = children(u, p).sorted(by: v => desc(down[v]))
    if cs.len == 0 and not guarded: land(u)
    for i in 1..cs.len:
        if i == 1 and not guarded: sweep(cs[0], Some(u), false)
        sweep(cs[i], Some(u), false)
        plan.push("B {u}")
    if guarded:
        if let heavy = cs.first:
            plan.push("M {u} {heavy}")
            sweep(heavy, Some(u), true)
    else:
        if cs.len == 1: sweep(cs[0], Some(u), false)
        plan.push("M {u} {p!}")

land(root)
sweep(root, None, true)
print(plan.len)
for s in plan: print(s)
