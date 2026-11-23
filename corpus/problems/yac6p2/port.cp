// Source: Yet Another Contest 6 P2 (No More Telemarketers), https://dmoj.ca/problem/yac6p2
// Problem: two rooted trees s, t on N <= 1e6 workers (boss or -1). A move re-hangs a worker A
//   under any B that A does not manage. Print the fewest moves turning s into t and the moves
//   as pairs (A, B), or -1 if impossible.
// Solution: the root manages everyone, so it never moves: different roots give -1. Otherwise
//   each worker whose boss differs moves exactly once, to its t-boss, in a preorder of t: the
//   new boss's chain to the root is already final, so it cannot pass through the worker. O(N).
// cp: 0.1 0f8d77b+dirty
let n: 1..=1e6 = read()
domain V = 1..=n

// a random tree, rooted at the first or (now and then) the last vertex
fn random_parents() -> [V]?V:
    if rand(0, 3) == 0: [V] v => (if v == V.last: None else: V(rand(v.label + 1, n)))
    else: [V] v => (if v == V.first: None else: V(rand(1, v.label - 1)))

let have: [V]?V = read(none: -1, gen: random_parents)
let want: [V]?V = read(none: -1, gen: random_parents)

fn root(parent: [V]?V) -> V: [v for v in V if parent[v] == None].min()!

if root(have) != root(want): print(-1)
else:
    // Re-hang top-down along the wanted tree: a vertex's new parent is then
    // already in place, so it is never hung beneath its own subtree.
    var kids: [V]List<V> = []
    for v in V:
        if let p = want[v]: kids[p].push(v)
    let moved = [v for v in Rooted(kids, root(want)).at if have[v] != want[v]]
    print(moved.len)
    for v in moved: print(v, want[v])
