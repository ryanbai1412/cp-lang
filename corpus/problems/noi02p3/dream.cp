// NOI 2002 Greedy Head: m heads eat all n fruits of a tree; the big head eats exactly k
// including the root, every head eats at least one. Pay an edge's weight when the big head eats
// both ends, and (only when m = 2) when the small head does. Minimise the cost, or -1.
// Tree knapsack: per vertex and whether the big head ate it, the cost by how many big-eaten
// vertices lie in its subtree; children merge by min-plus convolution.
let (n, m, k) = read (Int, Int, Int)
domain V = 1..=n
let t = Rooted(Graph.undirected(read [n - 1](u: V, v: V, w: Int)), root: V.first!)
if m - 1 + k > n: print(-1); return
enum Eater: Big, Small
fn pays(a: Eater, b: Eater) -> Bool = a == b and (a == Big or m == 2)

let cost = t.up((v, kids) =>
    [for me in Eater:
        kids.fold(if me == Big: [inf, 0] else: [0], (acc, (arc, sub)) =>
            convolve(acc, [for j in sub[Big].keys:
                min(for them in Eater: sub[them][j] + (if pays(me, them): arc.w else: 0))],
                over: Semiring(min, +)))])
print(cost[t.root][Big][k])
