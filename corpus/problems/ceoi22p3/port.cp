// Source: CEOI 2022 P3 (Prize), https://dmoj.ca/problem/ceoi22p3
// Problem: interactive. Two rooted trees on the same 2 <= N <= 1e6 labels with hidden edge
//   weights 1..=2000. Print 2 <= K <= 1e5 chosen labels on one line, ask Q questions (a, b),
//   each answered by the distances of a and b to their LCA in both trees; then answer
//   T <= min(K^2, 1e5) questions (p, q) on chosen labels with both distances, up to 2e9.
//   Q = K - 1 here, the full-score scheme (subtasks allow 2K - 2 or identical trees).
// Solution: choose the first K vertices of tree 1's preorder (closed under parents) and ask the
//   K - 1 pairs adjacent in tree 2's preorder. Each answer ties a and b to their LCA by a known
//   depth difference; in tree 2 those LCAs are the branch points of the chosen set's auxiliary
//   tree, in tree 1 they are chosen vertices, and both tie graphs are connected. One BFS per tree
//   fixes the depths up to a shift; then d(p, q) = dep p + dep q - 2 dep lca. O((N + T) log N).
// original: corpus/submissions/4728198.cpp
// Bounds: n: 2..=1_000_000; k: 2..=100_000; _, t: 1..=100_000; pairs.a, pairs.b: 1..=n

let (n, k, _, t): (2..=1_000_000, 2..=100_000, 1..=100_000, 1..=100_000) = read()
domain V = 1..=n
domain Ask = 0..k - 1

let one = Rooted.from_parents(read<[V]?V>(none: -1))
let two = Rooted.from_parents(read<[V]?V>(none: -1))

var picked: List<V> = [one.at[one.Pre.at(i)!] for i in 0..k]
print(picked)
picked.sort(by: v => two.tin[v].ord)
let ask_pairs = [(a, b) for (a, b) in picked.adjacent]
for (a, b) in ask_pairs: print("?", a, b)
print("!")

// ties[x]: (y, d) with depth[y] = depth[x] + d
fn depths(
    lca: (V, V) -> V,
    replies: [Ask](Int, Int, Int, Int),
    proj: ((Int, Int, Int, Int)) -> (Int, Int),
) -> [V]Int:
    var ties: [V]List<(to: V, d: Int)> = []
    for i in Ask:
        let (a, b) = ask_pairs[i.ord]
        let top = lca(a, b)
        let (da, db) = proj(replies[i])
        ties[top].push((to: a, d: da))
        ties[a].push((to: top, d: -da))
        ties[top].push((to: b, d: db))
        ties[b].push((to: top, d: -db))
    var depth: [V]?Int = None
    var queue: Deque<V> = Deque()
    depth[picked[0]] = 0
    queue.push(picked[0])
    while let x = queue.pop_front():
        let dx = depth[x]!
        for e in ties[x]:
            if depth[e.to] != None: continue
            depth[e.to] = dx + e.d
            queue.push(e.to)
    [V] v => depth[v] ?? 0

let replies: [Ask](Int, Int, Int, Int) = read()
let d1 = depths((a, b) => one.lca(a, b), replies, r => (r.0, r.1))
let d2 = depths((a, b) => two.lca(a, b), replies, r => (r.2, r.3))
domain Query = 0..t
let pairs: [Query](a: V, b: V) = read()
for p in pairs:
    print(
        d1[p.a] + d1[p.b] - 2 * d1[one.lca(p.a, p.b)],
        d2[p.a] + d2[p.b] - 2 * d2[two.lca(p.a, p.b)],
    )
