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
let n: 1..=1000 = read()
domain City = 1..=n

var adj: [City]List<City> = []
for _ in 0..n - 1:
    let (u, v): (City, City) = read()
    adj[u].push(v)
    adj[v].push(u)

var need: [City]Int = 0
fn children(u: City, p: ?City) -> List<City> = [v for v in adj[u] if Some(v) != p]

// detectives that clear the subtree of u entered from its parent p
fn label(u: City, p: ?City) -> Int:
    var (a, b) = (0, 0)  // the two largest child labels
    for v in children(u, p):
        let x = label(v, Some(u))
        if x > a: (a, b) = (x, a)
        elif x > b: b = x
    need[u] = if a == 0: 1 else: max(a, b + 1)
    need[u]

var root = City.first!
for r in City:
    if label(r, None) < label(root, None): root = r
print(label(root, None))

var plan: List<Str> = []
fn land(u: City): plan.push("L {u}")

// clear the subtree of u; `guarded`: a detective already stands on u and stays in the
// subtree, else one detective comes back up to p
fn sweep(u: City, p: ?City, guarded: Bool):
    var cs = children(u, p)
    cs.sort(by: v => -need[v])
    if cs.len == 0 and not guarded: land(u)
    for i in 1..cs.len:
        if i == 1 and not guarded: sweep(cs[0], Some(u), false)
        sweep(cs[i], Some(u), false)
        plan.push("B {u}")
    if guarded:
        if cs.len > 0:
            plan.push("M {u} {cs[0]}")
            sweep(cs[0], Some(u), true)
    else:
        if cs.len == 1: sweep(cs[0], Some(u), false)
        plan.push("M {u} {p!}")

land(root)
sweep(root, None, true)
print(plan.len)
for s in plan: print(s)
