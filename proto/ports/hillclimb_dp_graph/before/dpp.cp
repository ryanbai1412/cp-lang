// cp: 0.1 0f8d77b+dirty
// AtCoder DP P, Independent Set: paint each vertex of a tree white or black
// so that no two adjacent vertices are both black. Count the paintings.
const P = 1000000007
type Z = Mod<P>
let n: 1..=1e5 = read()
domain V = 1..=n
domain E = 1..n

fn random_tree() -> [E](u: V, v: V):
    [E] e => (V(rand(1, e.label)), V(e.label + 1))

let edges: [E](u: V, v: V) = read(gen: random_tree)
var adj: [V]List<V> = []
for (u, v) in edges:
    adj[u].push(v)
    adj[v].push(u)
let tree = Rooted(adj, V.first)

// ways(v, black): paintings of v's subtree given v's colour
@memo
fn ways(v: V, black: Bool) -> Z:
    product((if black: ways(c, false) else: ways(c, false) + ways(c, true)) for c in adj[v] if tree.parent(c) == v)

print(ways(V.first, false) + ways(V.first, true))
