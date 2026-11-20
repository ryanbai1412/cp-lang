// cp: 0.1 0f8d77b+dirty
// Yet Another Contest 6 P2: turn one rooted tree into another by moves that
// re-hang a vertex under a new parent; the root can never move. Print the
// moves, or -1.
let n: 1..=1e6 = read()
domain V = 1..=n

// a random tree, rooted at the first or (now and then) the last vertex
fn random_parents() -> [V]?V:
    if rand(0, 3) == 0: [V] v => (if v == V.last: none else: V(rand(v.label + 1, n)))
    else: [V] v => (if v == V.first: none else: V(rand(1, v.label - 1)))

let have: [V]?V = read(gen: random_parents)
let want: [V]?V = read(gen: random_parents)

fn root(parent: [V]?V) -> V: min(v for v in V if parent[v] == none)!

if root(have) != root(want): print(-1)
else:
    // Re-hang top-down along the wanted tree: a vertex's new parent is then
    // already in place, so it is never hung beneath its own subtree.
    var kids: [V]List<V> = []
    for v in V:
        if let p = want[v]: kids[p].push(v)
    let moved = [v for v in Rooted(kids, root(want)).preorder if have[v] != want[v]]
    print(moved.len)
    for v in moved: print(v, want[v])
