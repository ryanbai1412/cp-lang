// Source: NOI 2021 P1 (Light and Heavy Edges), https://dmoj.ca/problem/noi21p1
// limits: 1000 ms, 1024 MB
// Problem: a tree of n <= 1e5 vertices, every edge light. m <= 1e5 operations: "1 a b" makes
//   every edge with an endpoint on the a-b path light, then the path's edges heavy; "2 a b"
//   prints the number of heavy edges on the a-b path. Multitest, T <= 3.
// Solution: an edge is heavy iff both ends were last coloured by the same update, so an update
//   colours its path with its number and every vertex starts with a colour of its own. On a
//   heavy-light layout one lazy tree folds a range to its end colours and the number of equal
//   neighbours (heavy edges inside the range); a query adds those over its ranges and compares
//   colours across each range's top edge. O(m log^2 n).
// original: corpus/submissions/3859087.cpp
// Bounds: t: 1..=3; n, m: 1..=1e5; kind: 1..=2; a, b: 1..=n

// A range's colours at both ends and its count of equal neighbours; `l == 0` is the empty range.
type Run = (l: Int, r: Int, ties: Int)

fn join(a: Run, b: Run) -> Run:
    if a.l == 0: return b
    if b.l == 0: return a
    (a.l, b.r, a.ties + b.ties + (if a.r == b.l: 1 else: 0))

let blank: Run = (0, 0, 0)
let runs = Monoid(blank, join)
let paint = Action<Run, Int>(
    id: 0,
    compose: (new_c, old_c) => if new_c != 0: new_c else: old_c,
    apply: (c, run, span) => if c == 0: run else: (c, c, span - 1),
    over: runs,
)

let t: 1..=3 = read()
for _ in 0..t:
    let n: 1..=1e5 = read()
    let m: 1..=1e5 = read()
    domain V = 1..=n
    domain E = 1..n

    fn random_tree() -> [E](u: V, v: V):
        [E] e => (V(rand(1, e.label)), V(e.label + 1))

    let edges: [E](u: V, v: V) = read(gen: random_tree)
    let tree = Rooted(Graph.undirected(edges), V.first!)
    let own: [tree.Pre]Run = [tree.Pre] p => (-p.ord - 1, -p.ord - 1, 0)
    var colour = SegTree(runs, own, act: paint)
    var who: [tree.Pre]V = V.first!
    for v in V: who[tree.sub(v).first!] = v

    for now in 1..=m:
        let kind: 1..=2 = read()
        let a: V = read()
        let b: V = read()
        if kind == 1:
            for chain in tree.chains(a, b): colour.apply(chain.range, now)
        else:
            let peak = tree.sub(tree.lca(a, b)).first!
            var heavy = 0
            for chain in tree.chains(a, b):
                let l = chain.range.first!
                heavy += colour.fold(chain.range).ties
                // every range but the highest hangs from the path by one more edge
                if l != peak:
                    let above = tree.sub(tree.parent(who[l])!).first!
                    if colour[l].l == colour[above].l: heavy += 1
            print(heavy)
