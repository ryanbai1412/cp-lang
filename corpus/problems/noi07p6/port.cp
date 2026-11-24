// Source: NOI 2007 P6, https://dmoj.ca/problem/noi07p6
// limits: 600 ms, 256 MB
// Problem: a tree on 1 <= N <= 1000 cities hides a thief who moves arbitrarily fast; a
//   detective in his city or on his road catches him. Steps: L x lands a detective at x, B x
//   recalls one, M x y moves one along an edge. Print S, the fewest detectives that surely
//   catch him, the step count T <= 20000 and the plan.
// Solution: rooted at r, a subtree whose child subtrees need a >= b >= ... detectives needs
//   max(a, b + 1) (a leaf needs 1): clear the lighter children one by one while a guard holds
//   the vertex, then walk into the heaviest one. Reroot once to find each root's need, folding
//   the two largest branch needs. O(n).
//   Run the plan from the best root: land on it, clear lighter children and recall their
//   detectives, then step into the heaviest.
// original: corpus/submissions/3756630.cpp
// Bounds: n: 1..=1000; roads.u, roads.v: 1..=n
let n: 1..=1000 = read()
domain City = 1..=n
domain Road = 1..n
let roads: [Road](u: City, v: City) = read()
let g = Graph.undirected(roads)

// the two largest needs among some branches, (0, 0) for none
type Top = (Int, Int)
fn top2(x: Top, y: Top) -> Top = if x.0 >= y.0: (x.0, max(x.1, y.0)) else: (y.0, max(y.1, x.0))
let needs = Monoid((0, 0), top2)
fn cover(t: Top, _: City) -> Int = max(t.0, t.1 + 1)  // a leaf (0, 0) needs 1

let whole = Rooted(g, root: City.first!).reroot(needs, lift: (x, _, _) => (x, 0), finish: cover)
let root = City.min(by: r => whole[r])!
let tree = Rooted(g, root: root)
let down = tree.up(needs, lift: (x, _, _) => (x, 0), finish: cover)
print(down[root])

var plan: List<Str> = []
fn land(u: City): plan.push("L {u}")

// clear the subtree of u; `guarded`: a detective already stands on u and stays in the
// subtree, else one detective comes back up to p
fn sweep(u: City, p: ?City, guarded: Bool):
    let cs = tree.children(u).sorted(by: v => desc(down[v]))
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
