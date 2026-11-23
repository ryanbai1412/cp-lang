// btoi17p6 Cat in a Tree: the most marked vertices with every pair at distance >= d. Greedy from
// the leaves: a subtree reports its count and the distance down to its shallowest mark.
// Children merge deepest first, and a child's mark survives only if it is far enough from the
// shallowest one kept so far.
let (n, d) = read<(1..=200_000, 1..=200_000)>()
domain V = 0..n
let t = Rooted.from_parents(read<[V.drop(1)]V>())

fn place(v: V) -> (count: Int, near: Int):
    var count = 0
    var near = inf
    for (
        c,
        dist,
    ) in t.children(v).map(u => place(u)).map(r => (r.count, r.near + 1)).sorted(by: r => desc(r.1)):
        count += c - 1
        if dist + near >= d:
            count += 1
            near = dist
    if near >= d:
        (count + 1, 0)
    else:
        (count, near)
print(place(t.root).count)
