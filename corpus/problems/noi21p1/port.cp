// original: corpus/submissions/3859087.cpp
// NOI 2021 Heavy and Light Edges: a tree whose edges all start light.
// "1 a b": every edge touching the path from a to b turns light, then the
// edges on the path turn heavy. "2 a b": count the heavy edges on the path.
let t: 1..=3 = read()
for _ in 0..t:
    let n: 1..=1e5 = read()
    let m: 1..=1e5 = read()
    domain V = 1..=n
    domain E = 1..n

    fn random_tree() -> [E](u: V, v: V):
        [E] e => (V(rand(1, e.label)), V(e.label + 1))

    let edges: [E](u: V, v: V) = read(gen: random_tree)
    let tree = Rooted(edges, V.first)

    // Stamp the vertices of each "1" path with the number of the command.
    // An edge is heavy when its two ends carry the same stamp, because any
    // later path through just one of them stamps it differently.
    // The tree is laid out so that a path is a few ranges of cells. Inside
    // a range, also keep one bit per cell: is the edge to the cell before
    // heavy? Only where two ranges of a path meet are stamps compared.
    var stamp = AssignSum<tree.Pre>()
    var tied = AssignSum<tree.Pre>()
    var who: [tree.Pre]V = V.first
    for v in V: who[tree.sub(v).l] = v

    for now in 1..=m:
        let kind: 1..=2 = read()
        let a: V = read()
        let b: V = read()
        if kind == 1:
            for (l, r) in tree.chains(a, b):
                stamp.assign(l, r, now)
                tied.assign(l, r, 1)
                tied.assign(l, l, 0)
                if let after = r + 1: tied.assign(after, after, 0)
        else:
            let peak = tree.sub(tree.lca(a, b)).l
            var heavy = 0
            for (l, r) in tree.chains(a, b):
                if let second = l + 1 and l != r: heavy += tied.sum(second, r)
                // every range but the highest hangs from the path by one more edge
                if l != peak:
                    let above = tree.sub(tree.parent(who[l])!).l
                    if stamp.sum(l, l) != 0 and stamp.sum(l, l) == stamp.sum(above, above): heavy += 1
            print(heavy)
