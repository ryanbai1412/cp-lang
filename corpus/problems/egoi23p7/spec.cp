use std.graph.{complement, components}
// egoi23p7 Find the Tree: a spanning arborescence that uses every required arc and no
// forbidden one. Required arcs must already form a forest; each of its trees is one block that
// must hang from some other block. Block c can hang under block k unless every vertex of k is
// forbidden as the parent of c's root, so the blocks need a spanning arborescence of the
// complement of those "fully blocked" arcs: the root of the last tree a DFS of it starts
// reaches everything if anything does.
let (n, m1, m2) = read<(1..=300_000, 0..=300_000, 0..=300_000)>()
domain V = [n]
let must = read<[m1](child: V, parent: V)>()
let banned = read<[m2](child: V, parent: V)>()

fn solve() -> ?List<(child: V, parent: V)>:
    var up: [V]?V = None
    for e in must:
        if up[e.child] != None:
            return None
        up[e.child] = Some(e.parent)
    let (domain B, block) = components(Graph.undirected(must.map(e => (u: e.child, v: e.parent))))
    var top: [B]?V = None
    for v in V if up[v] == None:
        top[block[v]] = Some(v)
    if B.any(b => top[b] == None):
        return None
    let members = block.buckets()
    var forbidden: [V]HashSet<V> = HashSet()
    for e in banned:
        forbidden[e.child].add(e.parent)
    var blocked: [B]List<B> = []
    for c in B:
        for (k, hits) in forbidden[top[c]!].tally(by: v => block[v]).items:
            if hits == members[k].len:
                blocked[k].push(c)
    let hang = complement(blocked)
    // The last vertex a DFS finishes is the only block that can reach all the others.
    var root = B.first!
    for step in dfs(hang, from: B) if step is Exit(b):
        root = b
    let tree = bfs(hang, from: [root])
    if B.any(b => not tree.reached(b)):
        return None
    var out = List(must)
    for b in B if b != root:
        let k = tree.back[b]!.from
        let c = top[b]!
        out.push((child: c, parent: members[k].find(x => x not in forbidden[c])!))
    out

if let arcs = solve():
    for a in arcs:
        print(a.child, a.parent)
else:
    print("NO")
