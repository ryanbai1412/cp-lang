// NOI 2002 Greedy Head: m heads eat all n fruits of a tree; the big head eats exactly k
// including the root, every head eats at least one. Pay an edge's weight when the big head eats
// both ends, and (only when m = 2) when the small head does. Minimise the cost, or -1.
// Tree knapsack: per vertex and whether the big head ate it, the cost by how many big-eaten
// vertices lie in its subtree; children merge by min-plus convolution.
let (n, m, k) = read<(1..=300, 2..=300, 1..=300)>()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V, w: 0..=100_000)>()), root: V.first!)
enum Eater = Small | Big
let never = 1 << 40  // no way to eat it so; sums of a few hundred stay in range
fn pays(a: Eater, b: Eater) -> Bool = a == b and (a == Big or m == 2)

// cost(v)[me][j]: the cheapest subtree of v when `me` eats v and the big head eats j of it
fn cost(v: V) -> [Eater]List<Int>:
    var best: [Eater]List<Int> = [Eater] me => if me == Big: [never, 0] else: [0, never]
    for arc in t.child_arcs(v):
        let sub = cost(arc.to)
        best = [Eater] me =>
            let edge = (0..sub[Big].len).map(
                j => [Small, Big].map(
                    them => sub[them][j] + (if pays(me, them): arc.w else: 0)
                ).min()!
            ).collect()
            convolve(best[me], edge, over: Semiring(min, +), shape: Curved.Neither)
    best

print(if m - 1 + k > n: -1 else: cost(t.root)[Big][k])
