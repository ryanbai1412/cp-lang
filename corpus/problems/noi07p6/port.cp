// Source: NOI 2007 P6, https://dmoj.ca/problem/noi07p6
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
let n: 1..=1000 = read()
domain City = 1..=n
domain Road = 1..n
let roads: [Road](u: City, v: City) = read()
let g = Graph.undirected(roads, a => a.to)

var need: [City]Int = 0
fn children(u: City, p: ?City) -> List<City> = [v for v in g[u] if Some(v) != p]

// detectives that clear the subtree of u entered from its parent p
fn label(u: City, p: ?City) -> Int:
    let top = children(u, p).map(v => label(v, Some(u))).max_k(2).sorted(by: x => desc(x))
    need[u] = if top.len == 0: 1 elif top.len == 1: top[0] else: max(top[0], top[1] + 1)
    need[u]

let root = City.min(by: r => label(r, None))!
print(label(root, None))

var plan: List<Str> = []
fn land(u: City): plan.push("L {u}")

// clear the subtree of u; `guarded`: a detective already stands on u and stays in the
// subtree, else one detective comes back up to p
fn sweep(u: City, p: ?City, guarded: Bool):
    let cs = children(u, p).sorted(by: v => desc(need[v]))
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
