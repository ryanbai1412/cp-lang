// COCI 2021/22 C4 P4 Parkovi: put parks on exactly k vertices of a weighted tree to minimise
// the largest distance from any vertex to its nearest park. Print that distance and the parks.
domain V = [read Int]
let k = read Int
let t = Rooted(Graph.undirected(read [V.len - 1](u: V, v: V, w: Int)), root: V.first!)

// Greedy for radius r, bottom up: open a park at a child as soon as the uncovered vertex
// furthest below it would be out of reach from its parent.
fn parks(r: Int) -> List<V>:
    var open = []
    let state = t.up(leaf: v => (far: Some(0), near: inf), (v, kids) =>
        let near = kids.min(of: (a, s) => if a.to in open: a.w else: s.near + a.w) ?? inf
        var far = Some(0)
        for (a, s) in kids:
            if s.far.is_some_and(f => f + a.w > r):
                open.push(a.to)
            elif s.far is Some(f):
                far = Some(max(far!, f + a.w))
        (far: if far! + near <= r: None else: far, near))
    if state[t.root].far is Some(_): open.push(t.root)
    open

let r = first_true(0..=inf, r => parks(r).len <= k)!
let chosen = parks(r)
print(r)
print(chosen ++ V.filter(v => v not in chosen).take(k - chosen.len))
