// CEOI '22 P3 - Prize (25pt, interactive): two rooted trees on the same n
// vertices with hidden positive edge lengths. Pick k vertices (one per line),
// then ask k - 1 questions "? a b"; after "!" each is answered with four
// numbers: the distances from a and from b to their lowest common ancestor,
// in tree 1 and in tree 2. Then t pairs of picked vertices arrive and the
// distance of each pair in both trees must be printed.
//
// Pick the first k vertices of a preorder of tree 1 (closed under parents)
// and ask about adjacent pairs in tree 2's preorder. Those adjacent pairs'
// LCAs are the branching points of the auxiliary tree of the picked set in
// tree 2, and each answer ties two vertices by a known depth difference; in
// tree 1 the same answers tie picked vertices to picked ancestors. Both tie
// graphs are connected, so one BFS fixes every depth up to a shift, and each
// query distance is `depth[a] + depth[b] - 2 * depth[lca(a, b)]`.
// original: corpus/submissions/4728198.cpp

let (n, k, _, t): (2..=1_000_000, 2..=100_000, 1..=100_000, 1..=100_000) = read()
domain V = 1..=n
domain Ask = 0..k - 1

let one = Rooted.from_parents(read<[V]?V>(none: -1))
let two = Rooted.from_parents(read<[V]?V>(none: -1))

var picked: List<V> = [one.at[one.Pre.at(i)!] for i in 0..k]
// CEOI '22 judge expects the k chosen vertices one per line before queries.
for v in picked: print(v)
picked.sort(by: v => two.tin[v].ord)
let ask_pairs = [(a, b) for (a, b) in picked.pairs]
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
