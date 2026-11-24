// Tuned: flat child lists and one bottom-up pass for sizes and shapes instead of building a
//   `Rooted` and calling `shapes`, which costs most of the time on the largest input.
//
// Source: NOI 2022, Challenge NPC II, https://dmoj.ca/problem/noi22p4
// limits: 2000 ms, 1024 MB
// Problem: rooted trees G and H with |H| <= |G| <= |H| + k, k <= 5 (multitest, T <= 500,
//   |G| <= 1e5, sum |G| <= 5e5; first line C T k). Can deleting nodes of G leave a connected
//   part containing G's root that is isomorphic to H as a rooted tree? Print Yes or No.
// Solution: put both trees in one forest and name every subtree shape (`Rooted.shapes`).
//   prune(u, v), memoized on the shape pair: equal shapes match; else cancel the children of u
//   and v of equal shape (a Counter of shapes), then every leftover child of v needs its own
//   leftover child of u that prunes into it, a bipartite `max_matching` (u's other leftovers
//   are deleted whole). Each leftover costs a deletion, so at most k remain on either side.
// original: corpus/submissions/4802121.cpp
// Bounds: _c: 1..=26; tests: 1..=500; k: 0..=5; ng: 1..=1e5; g_par: -1..=ng; nh: 1..=ng; h_par: -1..=nh
with io(answers: ("Yes", "No"))

let _c: 1..=26 = read()
let tests: 1..=500 = read()
let k: 0..=5 = read()
for _ in 0..tests:
    let ng: 1..=1e5 = read()
    let g_par: [ng]-1..=ng = read()
    let nh: 1..=ng = read()
    let h_par: [nh]-1..=nh = read()
    domain Node = 0..ng + nh  // G's nodes, then H's
    fn up(p: Int, base: Int) -> ?Node = if p == -1: None else: Some(Node.nth(base + p - 1))
    let par = [Node] v => if v.ord < ng: up(g_par[v.ord], 0) else: up(h_par[v.ord - ng], ng)

    // children in one flat list: v's are kid[start[v]..start[v] + deg[v]]
    var deg: [Node]Int = 0
    for v in Node:
        if let p = par[v]: deg[p] += 1
    var start: [Node]Int = 0
    var filled = 0
    for v in Node:
        start[v] = filled
        filled += deg[v]
    var kid = List.filled(filled, Node.first!)
    var next_slot = start.clone()
    for v in Node:
        if let p = par[v]:
            kid[next_slot[p]] = v
            next_slot[p] += 1
    fn kids(v: Node) -> List<Node> = [kid[j] for j in start[v]..start[v] + deg[v]]

    // top-down order: both roots (G's first), then every node after its parent
    var order = [v for v in Node if par[v] == None]
    var i = 0
    while i < order.len:
        for j in start[order[i]]..start[order[i]] + deg[order[i]]: order.push(kid[j])
        i += 1

    // bottom-up: subtree sizes, and each subtree's shape named by its sorted children's shapes
    var size: [Node]Int = 1
    var shape: [Node]Int = 0
    var known = Map<List<Int>, Int>()
    for p in (0..order.len).rev:
        let v = order[p]
        if let up = par[v]: size[up] += size[v]
        var key = [shape[kid[j]] for j in start[v]..start[v] + deg[v]]
        key.sort()
        shape[v] = known.get_or_insert(key, known.len)

    // prune(u, v): can G's subtree u lose leaves until it is H's subtree v?
    var memo = Map<(Int, Int), Bool>()
    fn prune(u: Node, v: Node) -> Bool:
        if shape[u] == shape[v]: return true
        let spare = size[u] - size[v]
        if spare <= 0 or spare > k or deg[u] < deg[v]: return false
        if let known = memo.get((shape[u], shape[v])): return known
        var extra = Counter<Int>()
        for c in kids(u): extra.add(shape[c], 1)
        for c in kids(v): extra.add(shape[c], -1)
        var left: List<Node> = []
        for c in kids(u):
            if extra[shape[c]] > 0:
                left.push(c)
                extra.add(shape[c], -1)
        var right: List<Node> = []
        for c in kids(v):
            if extra[shape[c]] < 0:
                right.push(c)
                extra.add(shape[c], 1)
        // every leftover child of u costs a deletion: it shrinks into its own leftover child of v
        // or disappears whole
        var ok = left.len <= spare
        if ok:
            let fit = [[prune(have, want) for have in left] for want in right]
            domain Want = 0..right.len
            domain Have = 0..left.len
            ok = max_matching([Want] w => [h for h in Have if fit[w.ord][h.ord]]).size == right.len
        memo[(shape[u], shape[v])] = ok
        ok

    print(yes(prune(order[0], order[1])))
