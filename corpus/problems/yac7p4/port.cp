// Source: YAC 7 P4, https://dmoj.ca/problem/yac7p4
// limits: 3000 ms, 512 MB
// Problem: a tree of n <= 2e5 towns, one paparazzo in each. For each of m <= 2e5 appearances at a
//   town a, every paparazzo moves one road towards a (staying put at a); print the number of
//   occupied towns after each appearance.
// Solution: the occupied towns always form a connected subtree S. An appearance at a deletes
//   every leaf of S except the one met (a itself, or the town of S nearest a), and when a is
//   outside S adds the next town towards a. Root the tree: the top of S is tracked as it moves
//   (a deleted top hands over to its one neighbour, an added parent becomes the top). If a is
//   outside the top's subtree the new town is the top's parent; else it is the highest
//   unoccupied ancestor of a below the top, a `jump_while`. Leaves are kept as a set, and each
//   deletion pays for an insertion, so the total is O((n + m) log n).
// original: corpus/submissions/5705324.cpp
// Bounds: n, m: 1..=200_000; u, v, a: V
let (n, m): (1..=200_000, 1..=200_000) = read()
domain V = 1..=n
let roads: [0..n - 1](V, V) = read()
let tree = Rooted(Graph.undirected(roads), V.first!)

var occupied: [V]Bool = true
var size = n
var top = tree.root
var near: [V]Set<V> = []
for (u, v) in roads:
    near[u].insert(v)
    near[v].insert(u)
var leaves = Set([v for v in V if near[v].len == 1])

fn settle(v: V):
    if near[v].len == 1: leaves.insert(v) else: leaves.remove(v)

for a in read<[0..m]V>():
    if size == 1:
        print(1)
        continue
    let step: ?(V, V) =
        if occupied[a]: None
        elif not tree.is_anc(top, a): Some((top, tree.parent(top)!))
        else:
            let u = tree.jump_while(a, v => not occupied[v] and tree.is_anc(top, v))!
            Some((tree.parent(u)!, u))
    let met = if let (from, _) = step: from else: a
    for x in [x for x in leaves if x != met]:
        let y = near[x].min()!
        occupied[x] = false
        size -= 1
        leaves.remove(x)
        near[x].clear()
        near[y].remove(x)
        settle(y)
        if x == top: top = y
    if let (from, to) = step:
        occupied[to] = true
        size += 1
        near[from].insert(to)
        near[to].insert(from)
        settle(from)
        settle(to)
        if tree.parent(from) == Some(to): top = to
    print(size)
