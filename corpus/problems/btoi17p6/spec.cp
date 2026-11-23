// btoi17p6 Cat in a Tree: the most marked vertices with every pair at distance >= d. Greedy from
// the leaves: a subtree reports its count and the distance down to its shallowest mark.
// Children merge deepest first, and a child's mark survives only if it is far enough from the
// shallowest one kept so far.
let (n, d) = read<(1..=200_000, 1..=200_000)>()
domain V = 0..n
let up = read<[n - 1]V>()  // the parents of 1..n-1
let t = Rooted.from_parents([V] v => if v == V.first!: None else: Some(up[v.ord - 1]))

fn place(v: V) -> (count: Int, near: Int):
    let kids = t.children(v).map(u => place(u)).map(r => (count: r.count, dist: r.near + 1))
    var count = 0
    var near = inf
    for k in kids.sorted(by: k => desc(k.dist)):
        count += k.count - 1
        if near >= d - k.dist:
            count += 1
            near = k.dist
    if near >= d: (count + 1, 0) else: (count, near)
print(place(t.root).count)
